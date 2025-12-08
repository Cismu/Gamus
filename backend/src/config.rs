use gamus_scanner::config::ScannerConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerConfigDto {
  pub roots: Vec<String>,
  pub audio_exts: Vec<String>,
  pub ignore_hidden: bool,
  pub max_depth: Option<u32>,
}

impl From<ScannerConfig> for ScannerConfigDto {
  fn from(cfg: ScannerConfig) -> Self {
    ScannerConfigDto {
      roots: cfg.roots.into_iter().map(|p| p.to_string_lossy().to_string()).collect(),
      audio_exts: cfg.audio_exts,
      ignore_hidden: cfg.ignore_hidden,
      max_depth: cfg.max_depth,
    }
  }
}

impl From<ScannerConfigDto> for ScannerConfig {
  fn from(dto: ScannerConfigDto) -> Self {
    ScannerConfig {
      roots: dto.roots.into_iter().map(PathBuf::from).collect(),
      audio_exts: dto.audio_exts,
      ignore_hidden: dto.ignore_hidden,
      max_depth: dto.max_depth,
    }
  }
}
