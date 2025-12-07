// ports/scanner.rs
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
  #[error("io error: {0}")]
  Io(String),
}

pub trait FileScanner {
  fn scan_music_dirs(&self) -> Result<Vec<PathBuf>, ScanError>;
}
