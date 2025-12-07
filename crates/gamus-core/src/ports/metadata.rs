use std::path::Path;

use crate::domain::{release::Release, release_track::ReleaseTrack, song::Song};

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
  #[error("io error: {0}")]
  Io(String),

  #[error("unsupported format: {0}")]
  Unsupported(String),

  #[error("corrupt metadata: {0}")]
  Corrupt(String),

  #[error("missing mandatory tag: {0}")]
  Missing(String),

  #[error("internal error: {0}")]
  Internal(String),
}

/// Resultado de extraer metadatos de un archivo.
///
/// - `song`  → siempre presente (en el peor caso, derivado del filename)
/// - `release` → opcional (puede no haber álbum claro)
/// - `track`   → opcional (puede no haber track/disc number)
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
  pub song: Song,
  pub release: Option<Release>,
  pub track: Option<ReleaseTrack>,
}

/// Port que abstrae la lectura de metadatos desde un archivo de audio.
///
/// Implementaciones posibles:
/// - FFmpeg
/// - Lofty
/// - Symphonia
/// - combinaciones + servicios externos (MusicBrainz, etc.)
#[async_trait::async_trait]
pub trait MetadataExtractor: Send + Sync {
  async fn extract_from_path(&self, path: &Path) -> Result<ExtractedMetadata, MetadataError>;
}
