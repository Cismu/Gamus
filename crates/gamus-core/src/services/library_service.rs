use crate::domain::artist::Artist;
use crate::domain::release::Release;
use crate::domain::song::Song;
use crate::domain::{ArtistId, ReleaseId, SongId};
use crate::errors::CoreError;
use crate::ports::{Library, Probe, ProgressReporter, Scanner};

use futures::stream::{self, StreamExt};

/// Servicio de Aplicación para gestionar la Biblioteca.
///
/// Orquesta el escaneo, la extracción de metadatos y la persistencia.
/// Decide las políticas de concurrencia basándose en la información del Scanner.
pub struct LibraryService<S, M, R, P>
where
  S: Scanner + Clone,  // Necesitamos Clone para pasarlo a hilos si fuera necesario
  M: Probe + Clone,    // Necesitamos Clone para que cada hilo tenga su extractor
  R: Library + Clone,  // Necesitamos Clone para que cada hilo tenga su conexión a DB
  P: ProgressReporter, // El reporter suele ser un canal (mpsc) o Arc interno, no necesita Clone explícito aquí si es referencia compartida, pero Clone ayuda.
{
  scanner: S,
  metadata: M,
  repo: R,
  reporter: P,
}

impl<S, M, R, P> LibraryService<S, M, R, P>
where
  S: Scanner + Clone,
  M: Probe + Clone,
  R: Library + Clone,
  P: ProgressReporter,
{
  pub fn new(scanner: S, metadata: M, repo: R, reporter: P) -> Self {
    Self { scanner, metadata, repo, reporter }
  }

  /// Determina cuántos archivos procesar en paralelo basándose en la velocidad del disco.
  ///
  /// - NVMe (>500MB/s): 50 hilos (limitado por CPU para ffmpeg)
  /// - SSD/SATA (>100MB/s): 20 hilos
  /// - USB/Red/HDD (<100MB/s): 4 hilos (para evitar thrashing del cabezal o saturar bus)
  fn decide_concurrency(&self, mb_s_hint: Option<u64>) -> usize {
    match mb_s_hint {
      Some(speed) if speed > 500 => 50,
      Some(speed) if speed > 100 => 20,
      Some(_) => 4,
      None => 8, // Valor conservador por defecto
    }
  }

  /// Importa la biblioteca completa de manera asíncrona y reactiva.
  pub async fn import_full(&self) -> Result<(), CoreError> {
    // 1. ESCANEO: Obtener grupos de archivos (agrupados por dispositivo físico)
    //    Esto llama al puerto, que a su vez usa el adaptador de gamus-scanner
    let groups = self.scanner.scan_library_files().await.map_err(|e| CoreError::Scan(e.to_string()))?;

    // Calculamos el total global para inicializar la barra de progreso
    let total_files: usize = groups.iter().map(|g| g.files.len()).sum();
    self.reporter.start(total_files).await;

    // Preparamos referencias clonables de los servicios para inyectarlas en los closures async
    let meta_service_base = self.metadata.clone();
    let repo_service_base = self.repo.clone();

    // 2. PROCESAMIENTO: Iteramos grupo por grupo (Disco por Disco)
    //    Es importante procesar los discos de uno en uno para no saturar el sistema I/O global,
    //    pero dentro de cada disco, paralelizamos al máximo posible.
    for group in groups {
      // A) Decidir concurrencia para ESTE dispositivo
      let concurrency = self.decide_concurrency(group.device.bandwidth_mb_s);

      // B) Crear el Stream de procesamiento
      let mut stream = stream::iter(group.files)
        .map(|scanned_file| {
          // Clonamos 'handles' para esta tarea específica
          let meta = meta_service_base.clone();
          let repo = repo_service_base.clone();

          // El bloque async move captura las variables clonadas y el archivo
          async move {
            let path_str = scanned_file.path.to_string_lossy().to_string();

            // --- PASO 1: Extracción (CPU Bound / IO Read) ---
            let extracted = meta
              .extract_from_path(&scanned_file.path)
              .await
              .map_err(|e| (path_str.clone(), format!("Metadata error: {}", e)))?;

            // --- PASO 2: Persistencia (IO Write / DB) ---
            // Guardar Song
            repo.save_song(&extracted.song).map_err(|e| (path_str.clone(), format!("Repo song error: {}", e)))?;

            // Guardar Release (si existe)
            if let Some(release) = &extracted.release {
              repo.save_release(release).map_err(|e| (path_str.clone(), format!("Repo release error: {}", e)))?;
            }

            // Guardar Track / Relación (Pendiente de implementar en tus repos)
            // ...

            // Retornamos el path como éxito
            Ok::<String, (String, String)>(path_str)
          }
        })
        // C) BUFFER_UNORDERED: Aquí ocurre la magia de la concurrencia
        .buffer_unordered(concurrency);

      // D) CONSUMIR RESULTADOS: Mientras el buffer procesa, recibimos los resultados uno a uno
      while let Some(result) = stream.next().await {
        match result {
          Ok(path) => {
            self.reporter.on_success(&path).await;
          }
          Err((path, error_msg)) => {
            // Reportamos el error pero NO detenemos la importación
            self.reporter.on_error(&path, &error_msg).await;
          }
        }
      }
    }

    // 3. FINALIZAR
    self.reporter.finish().await;

    Ok(())
  }

  // -------- QUERIES (Lectura) --------
  // Estos métodos son simples pasamanos al repositorio

  pub fn list_artists(&self) -> Result<Vec<Artist>, CoreError> {
    self.repo.list_artists()
  }

  pub fn list_songs(&self) -> Result<Vec<Song>, CoreError> {
    self.repo.list_songs()
  }

  pub fn list_releases(&self) -> Result<Vec<Release>, CoreError> {
    self.repo.list_releases()
  }

  pub fn get_artist(&self, id: ArtistId) -> Result<Option<Artist>, CoreError> {
    self.repo.find_artist(id)
  }

  pub fn get_song(&self, id: SongId) -> Result<Option<Song>, CoreError> {
    self.repo.find_song(id)
  }

  pub fn get_release(&self, id: ReleaseId) -> Result<Option<Release>, CoreError> {
    self.repo.find_release(id)
  }
}
