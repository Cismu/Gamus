use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gamus_core::ports::scanner::{
  FileScanner as CoreFileScanner, ScanDevice, ScanError as CoreScanError, ScanGroup,
  ScannedFile as CoreScannedFile,
};

use crate::fs_scanner::{FsScanGroup, FsScannedFile, ScannerError, scan_groups_async};

/// Implementación de `FileScanner` para Gamus.
///
/// Mantiene una caché de velocidades de disco para evitar lecturas falsas
/// provocadas por el Page Cache del sistema operativo.
#[derive(Clone)]
pub struct GamusFileScanner {
  // Key: DeviceID (String), Value: Speed MB/s (u64)
  device_cache: Arc<Mutex<HashMap<String, u64>>>,
}

impl GamusFileScanner {
  pub fn new() -> Self {
    Self { device_cache: Arc::new(Mutex::new(HashMap::new())) }
  }
}

// Implementación de Default para facilitar la creación
impl Default for GamusFileScanner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl CoreFileScanner for GamusFileScanner {
  async fn scan_library_files(&self) -> Result<Vec<ScanGroup>, CoreScanError> {
    // 1. OBTENER SNAPSHOT DE LA CACHÉ
    // Bloqueamos el mutex solo el tiempo necesario para clonar el mapa.
    // No queremos mantener el bloqueo durante todo el escaneo (que es async).
    let known_speeds = {
      let guard = self
        .device_cache
        .lock()
        .map_err(|_| CoreScanError::Internal("Scanner mutex poisoned".to_string()))?;
      guard.clone()
    };

    // 2. LLAMAR AL SCANNER INTERNO (pasándole lo que ya sabemos)
    let groups = scan_groups_async(&known_speeds).await.map_err(map_scanner_error)?;

    // 3. ACTUALIZAR CACHÉ
    // Si el scanner detectó velocidades nuevas (porque eran devices nuevos),
    // las guardamos ahora para la próxima vez.
    {
      // Volvemos a bloquear para escribir
      if let Ok(mut guard) = self.device_cache.lock() {
        for g in &groups {
          if let Some(speed) = g.device.bandwidth_mb_s {
            guard.insert(g.device.id.clone(), speed);
          }
        }
      }
    }

    // 4. MAPEO (Infra -> Dominio)
    let mapped: Vec<ScanGroup> = groups
      .into_iter()
      .map(|g: FsScanGroup| {
        let device = ScanDevice { id: g.device.id, bandwidth_mb_s: g.device.bandwidth_mb_s };

        let files = g
          .files
          .into_iter()
          .map(|f: FsScannedFile| CoreScannedFile {
            path: f.path,
            size_bytes: f.size,
            modified_unix: f.modified,
          })
          .collect();

        ScanGroup { device, files }
      })
      .collect();

    Ok(mapped)
  }
}

fn map_scanner_error(err: ScannerError) -> CoreScanError {
  match err {
    ScannerError::Io(e) => CoreScanError::Io(e.to_string()),
    ScannerError::Walker(e) => CoreScanError::Internal(e),
    ScannerError::Config(e) => CoreScanError::Internal(e.to_string()),
  }
}
