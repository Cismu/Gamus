use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use futures::StreamExt;
use thiserror::Error;
use tokio::task;

use gamus_fs::async_walker::{Filtering, WalkConfig, walk_filtered};

use crate::config::ScannerConfig;
use crate::device::{device_id, measure_device_throughput};

#[derive(Debug, Error)]
pub enum ScannerError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("walker error: {0}")]
  Walker(String),

  #[error("config error: {0}")]
  Config(#[from] gamus_config::ConfigError),
}

/// Lightweight DTO representing a file found during scanning.
/// Minimal metadata is kept here to reduce memory footprint during large traversals.
#[derive(Debug, Clone)]
pub struct FsScannedFile {
  pub path: PathBuf,
  pub size: u64,
  pub modified: u64,
}

/// Represents a physical storage volume/partition.
/// Used to throttle or parallelize ingestion based on hardware capabilities.
#[derive(Debug, Clone)]
pub struct FsDevice {
  /// Unique identifier (e.g., UUID, mount point hash, or serial).
  pub id: String,
  /// Measured or cached read throughput in MB/s.
  pub bandwidth_mb_s: Option<u64>,
}

/// A collection of files located on a specific physical device.
/// This structure allows the consumer to schedule I/O tasks that respect physical disk boundaries.
#[derive(Debug, Clone)]
pub struct FsScanGroup {
  pub device: FsDevice,
  pub files: Vec<FsScannedFile>,
}

/// Checks if a file path corresponds to a supported audio format.
/// Comparisons are case-insensitive.
fn is_audio(path: &Path, cfg: &ScannerConfig) -> bool {
  let ext = match path.extension().and_then(|e| e.to_str()) {
    Some(e) => e.to_lowercase(),
    None => return false,
  };

  cfg.audio_exts.iter().any(|cfg_ext| cfg_ext.eq_ignore_ascii_case(&ext))
}

/// Safely extracts size and modification time.
/// Returns default UNIX epoch on systems where modification time is unavailable.
fn file_metadata(path: &Path) -> Result<(u64, u64), ScannerError> {
  let meta = fs::metadata(path)?;
  let size = meta.len();

  let modified = meta.modified()?.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

  Ok((size, modified))
}

pub async fn scan_music_from_config() -> Result<Vec<FsScannedFile>, ScannerError> {
  let cfg = ScannerConfig::load()?;
  scan_music_with_cfg(&cfg).await
}

/// Performs a recursive, asynchronous filesystem walk based on the provided configuration.
///
/// # Logic
/// * Uses `gamus_fs::async_walker` to stream directory entries without blocking the executor.
/// * Applies filtering for hidden files (optional in config) and temporary files (`.tmp`).
/// * Flattens the stream into a Vector.
///
/// # Performance Note
/// For libraries exceeding 100k files, the resulting `Vec` might cause a spike in heap allocation.
/// If memory constraints become an issue, refactor this to return a `Stream`.
pub async fn scan_music_with_cfg(cfg: &ScannerConfig) -> Result<Vec<FsScannedFile>, ScannerError> {
  let walk_cfg =
    WalkConfig { follow_symlinks: false, max_depth: cfg.max_depth.unwrap_or(50) as usize, dedup_dirs: true };

  let mut all_files = Vec::new();
  // Arc is required to share config across the stream's future boundary.
  let cfg_arc = Arc::new(cfg.clone());

  for root in &cfg_arc.roots {
    let cfg_for_root = Arc::clone(&cfg_arc);

    let entries = walk_filtered(root, walk_cfg.clone(), move |entry| {
      let path = entry.path.clone();
      let ignore_hidden = cfg_for_root.ignore_hidden;

      async move {
        // Security/UX: Skip hidden folders if configured to avoid scanning system directories.
        if ignore_hidden {
          if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with('.') {
              return Filtering::IgnoreDir;
            }
          }
        }

        // Ignore partial downloads or temp files common in sync folders.
        if path.extension().map_or(false, |e| e == "tmp") {
          return Filtering::Ignore;
        }

        Filtering::Continue
      }
    });

    tokio::pin!(entries);

    while let Some(res) = entries.next().await {
      let entry = match res {
        Ok(e) => e,
        Err(e) => {
          // Log but do not abort the entire scan on single permission errors.
          eprintln!("walker error: {e}");
          continue;
        }
      };

      let path = entry.path;

      if path.is_file() && is_audio(&path, &cfg_arc) {
        match file_metadata(&path) {
          Ok((size, modified)) => all_files.push(FsScannedFile { path, size, modified }),
          Err(e) => eprintln!("metadata error: {e}"),
        }
      }
    }
  }

  Ok(all_files)
}

/// Orchestrates the scanning process and groups files by their physical storage device.
///
/// # Architecture
/// This function is crucial for the ingestion strategy. By grouping files by device ID,
/// the consumer can determine whether to process groups in parallel (e.g., SSD + HDD)
/// or sequentially (single HDD) to avoid disk thrashing.
///
/// # Throughput Measurement
/// If `known_speeds` is missing an entry for a device, a micro-benchmark is triggered.
/// This IO operation is offloaded to `spawn_blocking` to prevent stalling the Tokio runtime.
pub async fn scan_groups_async(known_speeds: &HashMap<String, u64>) -> Result<Vec<FsScanGroup>, ScannerError> {
  let cfg = ScannerConfig::load()?;
  let files = scan_music_with_cfg(&cfg).await?;

  // 1) Group by device_id to isolate I/O domains.
  let mut by_device: HashMap<String, Vec<FsScannedFile>> = HashMap::new();

  for f in files {
    let dev_id = match device_id(&f.path) {
      Ok(id) => id,
      Err(e) => {
        // Fallback strategy: Treat unknown devices as a single generic group.
        eprintln!("device_id error for {}: {e}", f.path.display());
        "UNKNOWN_DEVICE".to_string()
      }
    };

    by_device.entry(dev_id).or_default().push(f);
  }

  const SAMPLE_BYTES: u64 = 20 * 1_048_576; // 20 MB sample for throughput test
  let mut handles = Vec::new();

  for (dev_id, files) in by_device {
    if let Some(&cached_speed) = known_speeds.get(&dev_id) {
      let files_clone = files;
      let dev_id_clone = dev_id.clone();

      // Fast path: Speed is known, just wrap in a future for uniform handling.
      let handle = tokio::spawn(async move { (dev_id_clone, Some(cached_speed), files_clone) });
      handles.push(handle);
    } else {
      let sample_path = files.get(0).map(|f| f.path.clone());

      // Slow path: Blocking I/O benchmark. Must be offloaded to thread pool.
      let handle = task::spawn_blocking(move || {
        let bw_opt = sample_path
          .as_ref()
          .and_then(|p| measure_device_throughput(p, SAMPLE_BYTES as usize).ok())
          .map(|bw| bw as u64);

        (dev_id, bw_opt, files)
      });
      handles.push(handle);
    }
  }

  let mut groups = Vec::new();

  for h in handles {
    let (dev_id, bw_opt, files) = h.await.map_err(|e| ScannerError::Walker(format!("join error: {e}")))?;

    let device = FsDevice { id: dev_id, bandwidth_mb_s: bw_opt };
    groups.push(FsScanGroup { device, files });
  }

  Ok(groups)
}
