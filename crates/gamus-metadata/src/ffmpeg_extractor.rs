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

/// Implementation of the `Probe` port using `ffmpeg-next` (FFmpeg FFI bindings).
///
/// **Architecture Note:**
/// Since `ffmpeg-next` relies on blocking C-interop calls, this struct is responsible
/// for strictly segregating these operations from the async runtime.
#[derive(Clone)]
pub struct FfmpegProbe;

impl FfmpegProbe {
  /// Initializes the underlying FFmpeg libraries.
  ///
  /// **Note:** Errors during initialization are logged via `eprintln` rather than
  /// panicking or returning `Result`. This design choice allows the application startup
  /// to proceed even if the media subsystem has partial failures, though functionality
  /// will be degraded.
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
  /// Extracts metadata from the given path.
  ///
  /// **Concurrency Strategy:**
  /// This method offloads the synchronous `extract_sync` function to `tokio::task::spawn_blocking`.
  /// This is critical to prevent starvation of the async reactor, as FFmpeg probing involves
  /// significant I/O and CPU overhead.
  async fn extract_from_path(&self, path: &Path) -> Result<ExtractedMetadata, MetadataError> {
    // Clone path to move ownership into the blocking closure.
    let path_buf = PathBuf::from(path);

    tokio::task::spawn_blocking(move || extract_sync(&path_buf))
      .await
      .map_err(|e| MetadataError::Internal(format!("Tokio Task Join Error: {}", e)))?
  }
}

/// Synchronous core logic for metadata extraction.
///
/// **Security Note:**
/// This function invokes FFmpeg on arbitrary input files. While `ffmpeg-next` provides
/// memory safety wrappers, the underlying C library processes complex, potentially
/// untrusted media formats. Ensure the runtime environment has appropriate resource
/// limits (OOM handling) in case of malformed files.
fn extract_sync(path: &Path) -> Result<ExtractedMetadata, MetadataError> {
  // 1. Filesystem Metadata
  // We capture FS metadata separately from media metadata for file-level tracking/syncing.
  let fs_metadata = std::fs::metadata(path).map_err(|e| MetadataError::Io(format!("Filesystem error: {}", e)))?;

  let modified_timestamp = fs_metadata
    .modified()
    .map_err(|e| MetadataError::Io(format!("Modified time unsupported: {}", e)))?
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();

  let file_details = FileDetails { path: path.to_path_buf(), size: fs_metadata.len(), modified: modified_timestamp };

  // 2. FFmpeg Context
  // Opens the container context. Failure here usually implies a corrupt file or unknown format.
  let context = ffmpeg::format::input(&path).map_err(|e| MetadataError::Io(format!("FFmpeg open failed: {}", e)))?;

  // Normalization: Keys are lowercased to simplify lookup logic downstream.
  let tags: HashMap<String, String> =
    context.metadata().iter().map(|(k, v)| (k.to_lowercase(), v.to_string())).collect();

  // --- SONG ---
  // Fallback Logic: If ID3 tags are missing, use the filename as the song title
  // to prevent empty UI states.
  let title = tags
    .get("title")
    .cloned()
    .unwrap_or_else(|| path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown Title").to_string());

  let song = Song { id: SongId::new(), title, acoustid: None };

  // --- RELEASE ---
  let album_title = tags.get("album").cloned().unwrap_or_else(|| "Unknown Album".to_string());

  // Date parsing is heuristic; we check standard date fields in order of likelihood.
  let date_str = tags.get("date").or_else(|| tags.get("year")).or_else(|| tags.get("original_year")).cloned();

  // Domain Logic: Genre vs Style
  // 1. Try to parse strictly into the `Genre` enum.
  // 2. If strict parsing fails, treat the string as a free-form `Style`.
  let genres = tags.get("genre").and_then(|s| Genre::from_str(s).ok()).map(|g| vec![g]).unwrap_or_default();

  let styles = if let Some(g_str) = tags.get("genre") {
    if genres.is_empty() { vec![Style::from_str(g_str).unwrap()] } else { Vec::new() }
  } else {
    Vec::new()
  };

  let release = Release {
    id: ReleaseId::new(),
    title: album_title,
    release_type: vec![ReleaseType::Album], // Defaulting to Album; inferred types require heuristic analysis not present here.
    main_artist_ids: Vec::new(),
    release_tracks: Vec::new(),
    release_date: date_str,
    artworks: Vec::new(),
    genres,
    styles,
  };

  // --- RELEASE TRACK ---
  // Heuristic: Handles "1" and "1/12" formats common in ID3v2.
  let track_number =
    tags.get("track").and_then(|t| t.split('/').next()).and_then(|n| n.parse::<u32>().ok()).unwrap_or(1);

  let disc_number = tags.get("disc").and_then(|d| d.split('/').next()).and_then(|d| d.parse::<u32>().ok()).unwrap_or(1);

  // Audio Details
  // Note: Duration in context is in AV_TIME_BASE units (microseconds).
  let duration_micros = context.duration();
  let duration = if duration_micros > 0 { Duration::from_micros(duration_micros as u64) } else { Duration::ZERO };

  let bitrate_kbps = if context.bit_rate() > 0 { Some((context.bit_rate() / 1000) as u32) } else { None };

  // --- STREAM DECODER ---
  // We instantiate a decoder for the best audio stream to extract specific codec parameters
  // (Sample Rate, Channels) that might not be reliably exposed in the container header.
  let audio_stream = context.streams().best(ffmpeg::media::Type::Audio);

  let (sample_rate_hz, channels) = if let Some(stream) = audio_stream {
    let params = stream.parameters();

    if let Ok(ctx) = ffmpeg::codec::context::Context::from_parameters(params) {
      if let Ok(audio_decoder) = ctx.decoder().audio() {
        (
          // Important: Use .rate() on the decoder context, not the container stream.
          Some(audio_decoder.rate()),
          Some(audio_decoder.channels() as u8),
        )
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
