use gamus_config::{CONFIG_BACKEND, ConfigError};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ScannerConfig {
  /// Directorios raíz a escanear en busca de música.
  pub roots: Vec<PathBuf>,
}

impl Default for ScannerConfig {
  fn default() -> Self {
    ScannerConfig {
      roots: Vec::new(), // vacío por defecto → la UI puede pedir que configures
    }
  }
}

impl ScannerConfig {
  pub fn load() -> Result<Self, ConfigError> {
    CONFIG_BACKEND.load_section_with_default("scanner")
  }
}
