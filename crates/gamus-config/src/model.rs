use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct FsConfig {
  pub audio_exts: Vec<String>,
  pub ignore_hidden: bool,
  pub max_depth: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
  pub db_filename: String,
  pub journal_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
  pub fs: FsConfig,
  pub storage: StorageConfig,
}
