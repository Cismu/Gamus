use gamus_config::{CONFIG_BACKEND, ConfigBackend, ConfigError, PATHS};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannerConfig {
  /// Directorios raíz a escanear.
  pub roots: Vec<PathBuf>,

  /// Extensiones de audio a considerar.
  #[serde(default = "default_audio_exts")]
  pub audio_exts: Vec<String>,

  /// Ignorar archivos/directorios ocultos.
  #[serde(default = "default_ignore_hidden")]
  pub ignore_hidden: bool,

  /// Profundidad máxima opcional.
  pub max_depth: Option<u32>,
}

fn default_audio_exts() -> Vec<String> {
  vec!["mp3".into(), "flac".into(), "ogg".into()]
}

fn default_ignore_hidden() -> bool {
  true
}

impl Default for ScannerConfig {
  fn default() -> Self {
    let mut roots = Vec::new();

    if let Some(audio_dir) = &PATHS.audio_dir {
      roots.push(audio_dir.clone());
    }

    if let Some(download_dir) = &PATHS.download_dir {
      roots.push(download_dir.clone());
    }

    ScannerConfig {
      roots,
      audio_exts: default_audio_exts(),
      ignore_hidden: default_ignore_hidden(),
      max_depth: None,
    }
  }
}

impl ScannerConfig {
  pub fn load() -> Result<Self, ConfigError> {
    let cfg = CONFIG_BACKEND.load_section_with_default("scanner")?;
    CONFIG_BACKEND.save_section("scanner", &cfg)?;
    Ok(cfg)
  }

  pub fn save(&self) -> Result<(), ConfigError> {
    CONFIG_BACKEND.save_section("scanner", self)
  }
}
