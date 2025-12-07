use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("toml error: {0}")]
  Toml(#[from] toml::de::Error),
  #[error("directories error: could not determine home directory")]
  Directories,
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
  pub fn new() -> Result<Self, ConfigError> {
    let (config_dir, data_dir, cache_dir, base_dir);

    if let Ok(env_base) = std::env::var("GAMUS_BASE_DIR") {
      let base = PathBuf::from(env_base);
      base_dir = base.clone();
      config_dir = base.join("config");
      data_dir = base.join("data");
      cache_dir = base.join("cache");
    } else {
      let proj_dirs = ProjectDirs::from("com", "gamus", "gamus").ok_or(ConfigError::Directories)?;
      base_dir = proj_dirs.config_dir().to_path_buf();
      config_dir = proj_dirs.config_dir().to_path_buf();
      data_dir = proj_dirs.data_dir().to_path_buf();
      cache_dir = proj_dirs.cache_dir().to_path_buf();
    }

    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(&cache_dir)?;

    Ok(Self { base_dir, config_dir, data_dir, cache_dir })
  }

  pub fn detect() -> Result<Self, ConfigError> {
    Self::new()
  }

  pub fn config_file(&self) -> PathBuf {
    self.config_dir.join("gamus.toml")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempdir;

  struct EnvVarGuard {
    key: String,
    original: Option<String>,
  }

  impl EnvVarGuard {
    fn new(key: &str, value: &str) -> Self {
      let original = std::env::var(key).ok();
      unsafe { std::env::set_var(key, value) };
      EnvVarGuard { key: key.to_owned(), original }
    }
  }

  impl Drop for EnvVarGuard {
    fn drop(&mut self) {
      match &self.original {
        Some(val) => unsafe { std::env::set_var(&self.key, val) },
        None => unsafe { std::env::remove_var(&self.key) },
      }
    }
  }

  #[test]
  fn test_gamus_base_dir_override() {
    let tmp = tempdir().unwrap();
    let _env = EnvVarGuard::new("GAMUS_BASE_DIR", tmp.path().to_str().unwrap());

    let paths = GamusPaths::new().unwrap();

    assert_eq!(paths.base_dir, tmp.path());
    assert_eq!(paths.config_dir, tmp.path().join("config"));
    assert_eq!(paths.data_dir, tmp.path().join("data"));
    assert_eq!(paths.cache_dir, tmp.path().join("cache"));

    assert!(paths.config_dir.exists());
    assert!(paths.data_dir.exists());
    assert!(paths.cache_dir.exists());
  }
}
