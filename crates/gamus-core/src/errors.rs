// crates/gamus-core/src/errors.rs
use thiserror::Error;

/// Error genérico del núcleo de Gamus.
///
/// Las capas superiores (Tauri, CLI, etc.) deberían mapear este error
/// a mensajes de usuario o logs.
#[derive(Debug, Error)]
pub enum CoreError {
  #[error("repository error: {0}")]
  Repository(String),

  #[error("scan error: {0}")]
  Scan(String),

  #[error("metadata error: {0}")]
  Metadata(String),

  #[error("not found")]
  NotFound,
  // Puedes ir afinando casos concretos a medida que avances
}
