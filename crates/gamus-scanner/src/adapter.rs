use gamus_core::ports::scanner::{
  FileScanner as CoreFileScanner, ScanDevice, ScanError as CoreScanError, ScanGroup,
  ScannedFile as CoreScannedFile,
};

use crate::fs_scanner::{FsScanGroup, FsScannedFile, ScannerError, scan_groups_sync};

/// Implementación de `FileScanner` para Gamus:
/// - usa config `[scanner]`
/// - recorre FS con `gamus-fs`
/// - agrupa por dispositivo y añade `bandwidth_mb_s`.
///
/// Internamente es async + spawn_blocking, pero hacia el dominio
/// expone una API **síncrona y limpia**.
pub struct GamusFileScanner;

impl GamusFileScanner {
  pub fn new() -> Self {
    GamusFileScanner
  }
}

impl CoreFileScanner for GamusFileScanner {
  fn scan_library_files(&self) -> Result<Vec<ScanGroup>, CoreScanError> {
    let groups = scan_groups_sync().map_err(map_scanner_error)?;

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
