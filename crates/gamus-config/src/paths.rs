use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("toml error: {0}")]
  Toml(#[from] toml::de::Error),
  #[error("other: {0}")]
  Other(String),
}

#[derive(Debug, Clone)]
pub struct GamusPaths {
  pub base_dir: PathBuf,
  pub config_dir: PathBuf,
  pub data_dir: PathBuf,
  pub cache_dir: PathBuf,
}

impl GamusPaths {
  pub fn detect() -> Result<Self, ConfigError> {
    // aquí puedes reciclar la lógica que tenías en CismuPaths:
    // - leer GAMUS_BASE_DIR / GAMUS_PORTABLE
    // - o usar directories::ProjectDirs
    // - crear carpetas si no existen
    unimplemented!()
  }

  pub fn config_file(&self) -> PathBuf {
    self.config_dir.join("gamus.toml")
  }
}
