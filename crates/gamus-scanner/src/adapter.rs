use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gamus_core::ports::scanner::{
  ScanDevice, ScanError as CoreScanError, ScanGroup, ScannedFile as CoreScannedFile, Scanner,
};

use crate::fs_scanner::{FsScanGroup, FsScannedFile, ScannerError, scan_groups_async};

/// Implementation of the `Scanner` port for local filesystem interactions.
///
/// This struct maintains state regarding physical device capabilities to optimize
/// ingestion strategies over the lifecycle of the application.
#[derive(Clone)]
pub struct FsScanner {
  /// Cache of device ID -> Throughput (MB/s).
  ///
  /// Wrapped in `Arc<Mutex>` to allow sharing the scanner instance across threads/tasks
  /// if necessary, though typical usage might be single-owner.
  /// We cache this to prevent re-triggering the blocking `measure_device_throughput`
  /// benchmark on every scan iteration.
  device_cache: Arc<Mutex<HashMap<String, u64>>>,
}

impl FsScanner {
  pub fn new() -> Self {
    Self { device_cache: Arc::new(Mutex::new(HashMap::new())) }
  }
}

impl Default for FsScanner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl Scanner for FsScanner {
  /// Orchestrates the scanning of local storage devices.
  ///
  /// # Concurrency Note
  /// This method employs a "snapshot-then-update" locking strategy. We acquire the
  /// cache lock strictly for reading/cloning initially, and release it *before*
  /// starting the I/O heavy `scan_groups_async`. This prevents holding the mutex
  /// during long-running asynchronous operations, avoiding potential contention.
  async fn scan_library_files(&self) -> Result<Vec<ScanGroup>, CoreScanError> {
    // 1. Snapshot known speeds.
    // Security: Handle poisoned mutexes gracefully by converting to an internal error.
    let known_speeds = {
      let guard =
        self.device_cache.lock().map_err(|_| CoreScanError::Internal("Scanner mutex poisoned".to_string()))?;
      guard.clone()
    };

    // 2. Perform the heavy I/O scan.
    // If a device is not in `known_speeds`, `scan_groups_async` will benchmark it.
    let groups = scan_groups_async(&known_speeds).await.map_err(map_scanner_error)?;

    // 3. Update cache with potential new benchmarks.
    // We re-acquire the lock to merge new data.
    {
      if let Ok(mut guard) = self.device_cache.lock() {
        for g in &groups {
          if let Some(speed) = g.device.bandwidth_mb_s {
            guard.insert(g.device.id.clone(), speed);
          }
        }
      }
    }

    // 4. Domain Adaptation.
    // Map infrastructure-layer DTOs (`FsScanGroup`) to Core Domain entities (`ScanGroup`).
    // This isolates the core from filesystem-specific implementation details (DTOs).
    let mapped: Vec<ScanGroup> = groups
      .into_iter()
      .map(|g: FsScanGroup| {
        let device = ScanDevice { id: g.device.id, bandwidth_mb_s: g.device.bandwidth_mb_s };

        let files = g
          .files
          .into_iter()
          .map(|f: FsScannedFile| CoreScannedFile { path: f.path, size_bytes: f.size, modified_unix: f.modified })
          .collect();

        ScanGroup { device, files }
      })
      .collect();

    Ok(mapped)
  }
}

/// Translates infrastructure-specific errors into domain-agnostic `CoreScanError`s.
///
/// This prevents leaking implementation details (e.g., specific walker crate errors)
/// up to the application layer.
fn map_scanner_error(err: ScannerError) -> CoreScanError {
  match err {
    ScannerError::Io(e) => CoreScanError::Io(e.to_string()),
    ScannerError::Walker(e) => CoreScanError::Internal(e),
    ScannerError::Config(e) => CoreScanError::Internal(e.to_string()),
  }
}
