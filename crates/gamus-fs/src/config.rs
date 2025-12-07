use gamus_config::{CONFIG_BACKEND, ConfigBackend, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FsConfig {
  pub audio_exts: Vec<String>,
  pub ignore_hidden: bool,
  pub max_depth: Option<u32>,
}

impl FsConfig {
  /// Carga desde la sección `[fs]` de gamus.toml usando el backend global.
  pub fn load() -> Result<Self, ConfigError> {
    CONFIG_BACKEND.load_section("fs")
  }

  /// Variante para tests: inyectar un backend distinto.
  pub fn load_from<B: ConfigBackend>(backend: &B) -> Result<Self, ConfigError> {
    backend.load_section("fs")
  }
}
