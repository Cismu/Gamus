use std::path::{Path, PathBuf};

use async_trait::async_trait;
use gamus_core::domain::release::Release;
use gamus_core::domain::release_type::ReleaseType;
use gamus_core::domain::{
  genre_styles::{Genre, Style},
  ids::{ArtistId, ReleaseId, ReleaseTrackId, SongId},
  release_track::ReleaseTrack,
  song::Song,
};
use gamus_core::ports::{ExtractedMetadata, MetadataError, MetadataExtractor};

pub struct FfmpegMetadataExtractor;

impl FfmpegMetadataExtractor {
  pub fn new() -> Self {
    Self
  }
}

impl Default for FfmpegMetadataExtractor {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl MetadataExtractor for FfmpegMetadataExtractor {
  async fn extract_from_path(&self, path: &Path) -> Result<ExtractedMetadata, MetadataError> {
    // ⚠️ Aquí dentro llamaremos a código bloqueante (FFmpeg, etc.)
    // así que lo envolvemos en spawn_blocking para no bloquear Tokio.
    let path = PathBuf::from(path);

    tokio::task::spawn_blocking(move || extract_sync(&path))
      .await
      .map_err(|e| MetadataError::Internal(format!("join error: {e}")))?
  }
}

/// Lógica síncrona real (aquí meterás FFmpeg luego).
fn extract_sync(path: &Path) -> Result<ExtractedMetadata, MetadataError> {
  // 1) Título de la canción
  let title = path
    .file_stem()
    .and_then(|s| s.to_str())
    .map(|s| s.to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

  let song = Song { id: SongId::new(), title, acoustid: None };

  // 2) Release opcional
  let release = build_release_from_parent_dir(path);

  // 3) Por ahora no construimos ReleaseTrack
  let track: Option<ReleaseTrack> = None;

  Ok(ExtractedMetadata { song, release, track })
}

fn build_release_from_parent_dir(path: &Path) -> Option<Release> {
  let parent: &Path = path.parent()?;
  let album_name = parent.file_name()?.to_str()?.to_string();

  Some(Release {
    id: ReleaseId::new(),
    title: album_name,
    release_type: vec![ReleaseType::Album],
    main_artist_ids: Vec::<ArtistId>::new(),
    release_tracks: Vec::<ReleaseTrackId>::new(),
    release_date: None,
    artworks: Vec::new(),
    genres: Vec::<Genre>::new(),
    styles: Vec::<Style>::new(),
  })
}
