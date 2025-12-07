// ports/metadata.rs
use crate::domain::{release::Release, release_track::ReleaseTrack, song::Song};

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
  #[error("io error: {0}")]
  Io(String),
  #[error("unsupported format")]
  Unsupported,
  // ...
}

pub trait MetadataExtractor {
  fn extract_from_path(
    &self,
    path: &std::path::Path,
  ) -> Result<(Song, Release, ReleaseTrack), MetadataError>;
}
