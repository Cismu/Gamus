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

/// Info mínima de un archivo escaneado (equivalente a FileDetails).
#[derive(Debug, Clone)]
pub struct FsScannedFile {
  pub path: PathBuf,
  pub size: u64,
  pub modified: u64,
}

/// Info de un dispositivo lógico (infra).
#[derive(Debug, Clone)]
pub struct FsDevice {
  pub id: String,
  pub bandwidth_mb_s: Option<u64>,
}

/// Grupo de resultados por dispositivo (infra).
#[derive(Debug, Clone)]
pub struct FsScanGroup {
  pub device: FsDevice,
  pub files: Vec<FsScannedFile>,
}

fn is_audio(path: &Path, cfg: &ScannerConfig) -> bool {
  let ext = match path.extension().and_then(|e| e.to_str()) {
    Some(e) => e.to_lowercase(),
    None => return false,
  };

  cfg.audio_exts.iter().any(|cfg_ext| cfg_ext.eq_ignore_ascii_case(&ext))
}

fn file_metadata(path: &Path) -> Result<(u64, u64), ScannerError> {
  let meta = fs::metadata(path)?;
  let size = meta.len();

  let modified = meta.modified()?.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

  Ok((size, modified))
}

/// Escanea los roots configurados y devuelve solo archivos de audio (async).
pub async fn scan_music_from_config() -> Result<Vec<FsScannedFile>, ScannerError> {
  let cfg = ScannerConfig::load()?;
  scan_music_with_cfg(&cfg).await
}

/// Versión async parametrizada con config: sólo detecta archivos.
pub async fn scan_music_with_cfg(cfg: &ScannerConfig) -> Result<Vec<FsScannedFile>, ScannerError> {
  let walk_cfg = WalkConfig {
    follow_symlinks: false,
    max_depth: cfg.max_depth.unwrap_or(50) as usize,
    dedup_dirs: true,
  };

  let mut all_files = Vec::new();
  let cfg_arc = Arc::new(cfg.clone());

  for root in &cfg_arc.roots {
    let cfg_for_root = Arc::clone(&cfg_arc);

    let entries = walk_filtered(root, walk_cfg.clone(), move |entry| {
      let path = entry.path.clone();
      let ignore_hidden = cfg_for_root.ignore_hidden;

      async move {
        if ignore_hidden {
          if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with('.') {
              return Filtering::IgnoreDir;
            }
          }
        }

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

/// Escaneo completo async:
///  - detecta archivos
///  - agrupa por dispositivo
///  - mide bandwidth por device con `spawn_blocking`.
pub async fn scan_groups_async() -> Result<Vec<FsScanGroup>, ScannerError> {
  let cfg = ScannerConfig::load()?;
  let files = scan_music_with_cfg(&cfg).await?;

  // 1) Agrupar por device_id
  let mut by_device: HashMap<String, Vec<FsScannedFile>> = HashMap::new();

  for f in files {
    let dev_id = match device_id(&f.path) {
      Ok(id) => id,
      Err(e) => {
        eprintln!("device_id error for {}: {e}", f.path.display());
        "UNKNOWN_DEVICE".to_string()
      }
    };

    by_device.entry(dev_id).or_default().push(f);
  }

  // 2) spawn_blocking por device para medir throughput
  let sample_bytes = 3 * 1_048_576; // 3 MiB
  let mut handles = Vec::new();

  for (dev_id, files) in by_device {
    let sample_path = files.get(0).map(|f| f.path.clone());

    let handle = task::spawn_blocking(move || {
      let bw_opt = sample_path
        .as_ref()
        .and_then(|p| measure_device_throughput(p, sample_bytes).ok())
        .map(|bw| bw as u64);

      (dev_id, bw_opt, files)
    });

    handles.push(handle);
  }

  // 3) Recolectar resultados
  let mut groups = Vec::new();

  for h in handles {
    let (dev_id, bw_opt, files) =
      h.await.map_err(|e| ScannerError::Walker(format!("join error: {e}")))?;
    let device = FsDevice { id: dev_id, bandwidth_mb_s: bw_opt };
    groups.push(FsScanGroup { device, files });
  }

  Ok(groups)
}
