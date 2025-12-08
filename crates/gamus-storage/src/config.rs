use gamus_config::{CONFIG_BACKEND, ConfigBackend, ConfigError, PATHS};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
  pub db_path: PathBuf,
  pub journal_mode: Option<String>,
}

impl Default for StorageConfig {
  fn default() -> Self {
    let db_path = PATHS.data_dir.join("gamus.db");
    StorageConfig { db_path, journal_mode: Some("WAL".to_string()) }
  }
}

impl StorageConfig {
  pub fn load() -> Result<Self, ConfigError> {
    let cfg = CONFIG_BACKEND.load_section_with_default("storage")?;
    CONFIG_BACKEND.save_section("storage", &cfg)?;
    Ok(cfg)
  }

  pub fn save(&self) -> Result<(), ConfigError> {
    CONFIG_BACKEND.save_section("storage", self)
  }
}
