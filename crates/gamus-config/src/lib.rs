mod backend;
mod paths;

pub use backend::{ConfigBackend, TomlConfigBackend};
pub use paths::{ConfigError, GamusPaths};

use once_cell::sync::Lazy;

// Singleton de paths (portable / system)
pub static PATHS: Lazy<GamusPaths> = Lazy::new(|| GamusPaths::detect().expect("failed to init GamusPaths"));

// Singleton del backend de config
pub static CONFIG_BACKEND: Lazy<TomlConfigBackend> = Lazy::new(|| TomlConfigBackend::new(PATHS.clone()));
