use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use futures::StreamExt;
use thiserror::Error;

use crate::async_walker::{Filtering, WalkConfig, walk_filtered};
use crate::config::FsConfig;

#[derive(Debug, Error)]
pub enum FsError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("walker error: {0}")]
  Walker(String),

  #[error("config error: {0}")]
  Config(#[from] gamus_config::ConfigError),
}

/// Info mínima que nos interesa de un archivo de audio.
///
/// Esto mapea 1:1 con `FileDetails` de tu dominio:
/// - path        -> PathBuf
/// - size_bytes  -> u64
/// - modified    -> u64 (unix seconds)
#[derive(Debug, Clone)]
pub struct ScannedFile {
  pub path: PathBuf,
  pub size: u64,
  pub modified: u64,
}

/// Extensiones que consideramos “audio” por ahora.

fn is_audio(path: &Path, cfg: &FsConfig) -> bool {
  let ext = match path.extension().and_then(|e| e.to_str()) {
    Some(e) => e.to_lowercase(),
    None => return false,
  };

  cfg.audio_exts.iter().any(|cfg_ext| cfg_ext.eq_ignore_ascii_case(&ext))
}

fn file_metadata(path: &Path) -> Result<(u64, u64), FsError> {
  let meta = fs::metadata(path)?;
  let size = meta.len();

  let modified = meta.modified()?.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

  Ok((size, modified))
}

/// Escanea el árbol y devuelve **solo archivos de audio**, con info básica.
///
/// Esto es ideal para luego mapear a `FileDetails` + `library_files`.
pub async fn scan_music_files(root: &str) -> Result<Vec<ScannedFile>, FsError> {
  let cfg = FsConfig::load()?;

  let walk_cfg = WalkConfig {
    follow_symlinks: false,
    max_depth: cfg.max_depth.unwrap_or(50) as usize,
    dedup_dirs: true,
  };

  let entries = walk_filtered(root, walk_cfg, |entry| {
    let path = entry.path.clone();

    async move {
      // Ignorar directorios ocultos
      if let Some(name) = path.file_name() {
        if name.to_string_lossy().starts_with('.') {
          return Filtering::IgnoreDir;
        }
      }

      // Ignorar basura temporal
      if path.extension().map_or(false, |e| e == "tmp") {
        return Filtering::Ignore;
      }

      Filtering::Continue
    }
  });

  tokio::pin!(entries);

  let mut files = Vec::new();

  while let Some(res) = entries.next().await {
    let entry = match res {
      Ok(e) => e,
      Err(e) => {
        eprintln!("walker error: {e}");
        continue;
      }
    };

    let path = entry.path;

    if path.is_file() && is_audio(&path, &cfg) {
      match file_metadata(&path) {
        Ok((size, modified)) => files.push(ScannedFile { path, size, modified }),
        Err(e) => {
          eprintln!("metadata error: {e}");
        }
      }
    }
  }

  Ok(files)
}
