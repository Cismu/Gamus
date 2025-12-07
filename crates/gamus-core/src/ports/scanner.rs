use std::path::PathBuf;

/// Información básica de un archivo detectado por el scanner.
///
/// Esto es “lo que el dominio necesita” para luego mapear a `FileDetails`
/// y decidir qué hacer con el archivo.
#[derive(Debug, Clone)]
pub struct ScannedFile {
  pub path: PathBuf,
  pub size_bytes: u64,
  pub modified_unix: u64,
}

/// Información de un dispositivo lógico donde se encontraron archivos.
///
/// No define el formato de `id`: eso es decisión del adapter.
/// Puede ser `dev_t` en Unix, `C:` en Windows, etc.
#[derive(Debug, Clone)]
pub struct ScanDevice {
  pub id: String,
  /// Ancho de banda aproximado del dispositivo (MB/s), si el adapter lo mide.
  pub bandwidth_mb_s: Option<u64>,
}

/// Grupo de archivos pertenecientes al mismo dispositivo.
///
/// Esto permite al dominio:
/// - tomar decisiones por dispositivo (paralelismo, prioridades),
/// - mostrar estadísticas por disco, etc.
#[derive(Debug, Clone)]
pub struct ScanGroup {
  pub device: ScanDevice,
  pub files: Vec<ScannedFile>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
  #[error("io error: {0}")]
  Io(String),

  #[error("internal error: {0}")]
  Internal(String),
}

/// Port de scanner de archivos de biblioteca.
///
/// No expone detalles de implementación (Tokio, async, etc.). El adapter
/// puede ser hiper-paralelo por dentro, pero desde el dominio se ve como
/// una operación síncrona que devuelve los resultados ya agrupados.
pub trait FileScanner {
  fn scan_library_files(&self) -> Result<Vec<ScanGroup>, ScanError>;
}
