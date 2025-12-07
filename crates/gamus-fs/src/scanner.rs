use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use thiserror::Error;

use crate::async_walker::{Filtering, WalkConfig, walk_filtered};

#[derive(Debug, Error)]
pub enum FsError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("walker error: {0}")]
  Walker(String),
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
fn is_audio(path: &Path) -> bool {
  const AUDIO_EXTS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "opus"];

  match path.extension().and_then(|e| e.to_str()) {
    Some(ext) => AUDIO_EXTS.contains(&ext.to_lowercase().as_str()),
    None => false,
  }
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
  let cfg = WalkConfig { follow_symlinks: false, max_depth: 64, dedup_dirs: true };

  let entries = walk_filtered(root, cfg, |entry| {
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

    if path.is_file() && is_audio(&path) {
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
