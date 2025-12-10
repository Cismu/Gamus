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

/// Escaneo completo async con "Memoización" de velocidad:
///  - Detecta archivos.
///  - Agrupa por dispositivo.
///  - Si el dispositivo YA está en `known_speeds`, usa ese valor (evita cache hit falso).
///  - Si es nuevo, mide con `spawn_blocking`.
pub async fn scan_groups_async(
  known_speeds: &HashMap<String, u64>,
) -> Result<Vec<FsScanGroup>, ScannerError> {
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

  // 2) Preparar tareas de medición
  // Aumentamos la muestra a 20 MB para amortiguar lecturas falsas
  const SAMPLE_BYTES: u64 = 20 * 1_048_576;
  let mut handles = Vec::new();

  for (dev_id, files) in by_device {
    // CASO A: Ya conocemos la velocidad de este dispositivo (está en caché)
    // No volvemos a medir para evitar lecturas "hot" de RAM a 1000MB/s
    if let Some(&cached_speed) = known_speeds.get(&dev_id) {
      // Creamos una tarea dummy que retorna inmediatamente el valor conocido
      let files_clone = files; // Movemos ownership
      let dev_id_clone = dev_id.clone();

      let handle = tokio::spawn(async move { (dev_id_clone, Some(cached_speed), files_clone) });
      handles.push(handle);
    }
    // CASO B: Dispositivo nuevo, hay que medir "en frío"
    else {
      let sample_path = files.get(0).map(|f| f.path.clone());

      // Usamos spawn_blocking porque leer 20MB bloquea el hilo
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

  // 3) Recolectar resultados
  let mut groups = Vec::new();

  for h in handles {
    // Handle join error
    let (dev_id, bw_opt, files) =
      h.await.map_err(|e| ScannerError::Walker(format!("join error: {e}")))?;

    let device = FsDevice { id: dev_id, bandwidth_mb_s: bw_opt };
    groups.push(FsScanGroup { device, files });
  }

  Ok(groups)
}
