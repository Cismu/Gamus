use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use ffmpeg_next as ffmpeg;

use gamus_core::domain::release::Release;
use gamus_core::domain::release_type::ReleaseType;
use gamus_core::domain::{
  genre_styles::{Genre, Style},
  ids::{ReleaseId, ReleaseTrackId, SongId},
  release_track::{AudioDetails, FileDetails, ReleaseTrack},
  song::Song,
};
use gamus_core::ports::{ExtractedMetadata, MetadataError, Probe};

use crate::mapping::*;

#[derive(Clone)]
pub struct FfmpegProbe;

impl FfmpegProbe {
  pub fn new() -> Self {
    if let Err(e) = ffmpeg::init() {
      eprintln!("Aviso: Error inicializando FFmpeg: {}", e);
    }
    Self
  }
}

impl Default for FfmpegProbe {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl Probe for FfmpegProbe {
  async fn extract_from_path(&self, path: &Path) -> Result<ExtractedMetadata, MetadataError> {
    let path_buf = PathBuf::from(path);

    // Ejecutamos la lógica síncrona/bloqueante en un hilo separado de Tokio
    tokio::task::spawn_blocking(move || extract_sync(&path_buf))
      .await
      .map_err(|e| MetadataError::Internal(format!("Tokio Task Join Error: {}", e)))?
  }
}

/// Lógica principal de extracción síncrona
fn extract_sync(path: &Path) -> Result<ExtractedMetadata, MetadataError> {
  let fs_metadata = std::fs::metadata(path).map_err(|e| MetadataError::Io(format!("Filesystem error: {}", e)))?;

  let modified_timestamp = fs_metadata
    .modified()
    .map_err(|e| MetadataError::Io(format!("Modified time unsupported: {}", e)))?
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();

  let file_details = FileDetails { path: path.to_path_buf(), size: fs_metadata.len(), modified: modified_timestamp };

  let context = ffmpeg::format::input(&path).map_err(|e| MetadataError::Io(format!("FFmpeg open failed: {}", e)))?;

  let tags: HashMap<String, String> =
    context.metadata().iter().map(|(k, v)| (k.to_lowercase(), v.to_string())).collect();

  let title = find_tag_value(&tags, KEYS_TITLE)
    .unwrap_or_else(|| path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown Title").to_string());

  let song = Song { id: SongId::new(), title, acoustid: None };

  let album_title = find_tag_value(&tags, KEYS_ALBUM).unwrap_or_else(|| "Unknown Album".to_string());

  let date_str = find_tag_value(&tags, KEYS_DATE);

  let raw_genre = find_tag_value(&tags, KEYS_GENRE);

  let (genres, styles) = if let Some(ref s) = raw_genre {
    match Genre::from_str(s) {
      Ok(g) => (vec![g], Vec::new()),
      Err(_) => (Vec::new(), vec![Style::from_str(s).unwrap()]),
    }
  } else {
    (Vec::new(), Vec::new())
  };

  let release = Release {
    id: ReleaseId::new(),
    title: album_title,
    release_type: vec![ReleaseType::Album], // Default, heurística pendiente
    main_artist_ids: Vec::new(),
    release_tracks: Vec::new(),
    release_date: date_str,
    artworks: Vec::new(),
    genres,
    styles,
  };

  // Track Number: Busca en alias y divide "1/12" si es necesario
  let track_number = find_tag_value(&tags, KEYS_TRACK_NUMBER)
    .and_then(|t| t.split('/').next().map(|s| s.to_string()))
    .and_then(|n| n.parse::<u32>().ok())
    .unwrap_or(1);

  // Disc Number: Busca en alias y divide "1/2" si es necesario
  let disc_number = find_tag_value(&tags, KEYS_DISC_NUMBER)
    .and_then(|d| d.split('/').next().map(|s| s.to_string()))
    .and_then(|d| d.parse::<u32>().ok())
    .unwrap_or(1);

  // Audio Details y Streams
  let duration_micros = context.duration();
  let duration = if duration_micros > 0 { Duration::from_micros(duration_micros as u64) } else { Duration::ZERO };

  let bitrate_kbps = if context.bit_rate() > 0 { Some((context.bit_rate() / 1000) as u32) } else { None };

  // Decoder Logic para Sample Rate y Channels
  let audio_stream = context.streams().best(ffmpeg::media::Type::Audio);
  let (sample_rate_hz, channels) = if let Some(stream) = audio_stream {
    let params = stream.parameters();
    if let Ok(ctx) = ffmpeg::codec::context::Context::from_parameters(params) {
      if let Ok(audio_decoder) = ctx.decoder().audio() {
        (Some(audio_decoder.rate()), Some(audio_decoder.channels() as u8))
      } else {
        (None, None)
      }
    } else {
      (None, None)
    }
  } else {
    (None, None)
  };

  let audio_details =
    AudioDetails { duration, bitrate_kbps, sample_rate_hz, channels, analysis: None, fingerprint: None };

  let track = ReleaseTrack {
    id: ReleaseTrackId::new(),
    song_id: song.id,
    release_id: release.id,
    track_number,
    disc_number,
    title_override: None,
    artist_credits: Vec::new(),
    audio_details,
    file_details,
  };

  Ok(ExtractedMetadata { song, release: Some(release), track: Some(track) })
}
