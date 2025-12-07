use gamus_config::PATHS;
use gamus_config::{CONFIG_BACKEND, ConfigError};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
  pub db_filename: String,
  pub journal_mode: Option<String>,
}

impl Default for StorageConfig {
  fn default() -> Self {
    StorageConfig { db_filename: "gamus.db".to_string(), journal_mode: Some("WAL".to_string()) }
  }
}

impl StorageConfig {
  pub fn load() -> Result<Self, ConfigError> {
    // usa la versión con defaults; si no hay archivo o falta [storage],
    // te devuelve StorageConfig::default().
    CONFIG_BACKEND.load_section_with_default("storage")
  }

  /// Ruta completa al archivo de DB según `gamus-config` paths.
  pub fn db_path(&self) -> PathBuf {
    PATHS.data_dir.join(&self.db_filename)
  }
}
