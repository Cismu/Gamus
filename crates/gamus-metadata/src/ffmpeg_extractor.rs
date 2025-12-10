use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};

use async_trait::async_trait;
use ffmpeg_next as ffmpeg;

use gamus_core::domain::release::Release;
use gamus_core::domain::release_track::{AudioAnalysis, AudioQuality};
use gamus_core::domain::release_type::ReleaseType;
use gamus_core::domain::{
  genre_styles::{Genre, Style},
  ids::{ReleaseId, ReleaseTrackId, SongId},
  release_track::{AudioDetails, FileDetails, ReleaseTrack},
  song::Song,
};
use gamus_core::ports::{ExtractedMetadata, MetadataError, Probe};

use crate::spectral_analyzer::{AnalysisConfig, SpectralAnalyzer};
use crate::tag_keys::*;

/// Adaptador FFmpeg que implementa el port `Probe`.
///
/// - Se mantiene completamente en la capa de infraestructura.
/// - No expone tipos de FFmpeg hacia el dominio.
/// - El análisis espectral es opcional y configurable.
#[derive(Clone)]
pub struct FfmpegProbe {
  analysis_config: Option<AnalysisConfig>,
}

impl FfmpegProbe {
  pub fn new_with_analysis(config: AnalysisConfig) -> Self {
    if let Err(e) = ffmpeg::init() {
      // Log deliberado: no abortamos, pero queremos visibilidad en entorno de servidor.
      eprintln!("Aviso: error inicializando FFmpeg: {e}");
    }

    Self { analysis_config: Some(config) }
  }

  pub fn new_without_analysis() -> Self {
    if let Err(e) = ffmpeg::init() {
      eprintln!("Aviso: error inicializando FFmpeg: {e}");
    }

    Self { analysis_config: None }
  }
}

impl Default for FfmpegProbe {
  fn default() -> Self {
    Self::new_with_analysis(AnalysisConfig::default())
  }
}

#[async_trait]
impl Probe for FfmpegProbe {
  async fn extract_from_path(&self, path: &Path) -> Result<ExtractedMetadata, MetadataError> {
    let path_buf = PathBuf::from(path);
    let analysis_config = self.analysis_config.clone();

    // Toda la parte bloqueante (FFmpeg + FFT) se delega a un hilo de trabajo.
    tokio::task::spawn_blocking(move || extract_sync(&path_buf, analysis_config))
      .await
      .map_err(|e| MetadataError::Internal(format!("Tokio task join error: {e}")))?
  }
}

/// Lógica principal síncrona, pensada para correrse en `spawn_blocking`.
fn extract_sync(path: &Path, analysis_config: Option<AnalysisConfig>) -> Result<ExtractedMetadata, MetadataError> {
  let file_details = build_file_details(path)?;
  let mut context = open_ffmpeg_input(path)?;

  let tags = collect_normalized_tags(&context);

  let song = build_song(path, &tags);
  let release = build_release(&tags)?;
  let (duration, bitrate_kbps) = extract_container_level_audio_info(&context);
  let (sample_rate_hz, channels) = extract_stream_level_audio_info(&mut context);
  let quality = run_spectral_analysis(path, analysis_config)?;
  let analysis = AudioAnalysis { bpm: None, features: None, quality };

  let audio_details =
    AudioDetails { duration, bitrate_kbps, sample_rate_hz, channels, analysis: Some(analysis), fingerprint: None };

  let track = build_release_track(&song, &release, &tags, audio_details, file_details);

  Ok(ExtractedMetadata { song, release: Some(release), track: Some(track) })
}

// ----- helpers de alto nivel ------------

fn build_file_details(path: &Path) -> Result<FileDetails, MetadataError> {
  let fs_metadata = std::fs::metadata(path).map_err(|e| MetadataError::Io(format!("filesystem error: {e}")))?;

  let modified_timestamp = fs_metadata
    .modified()
    .map_err(|e| MetadataError::Io(format!("modified time unsupported: {e}")))?
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();

  Ok(FileDetails { path: path.to_path_buf(), size: fs_metadata.len(), modified: modified_timestamp })
}

fn open_ffmpeg_input(path: &Path) -> Result<ffmpeg::format::context::Input, MetadataError> {
  ffmpeg::format::input(path).map_err(|e| MetadataError::Unsupported(format!("FFmpeg open failed: {e}")))
}

fn collect_normalized_tags(context: &ffmpeg::format::context::Input) -> HashMap<String, String> {
  context.metadata().iter().map(|(k, v)| (k.to_lowercase(), v.to_string())).collect()
}

fn build_song(path: &Path, tags: &HashMap<String, String>) -> Song {
  let title = find_tag_value(tags, KEYS_TITLE)
    .map(|s| s.to_string())
    .or_else(|| path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))
    .unwrap_or_else(|| "Unknown Title".to_string());

  Song { id: SongId::new(), title, acoustid: None }
}

fn build_release(tags: &HashMap<String, String>) -> Result<Release, MetadataError> {
  let album_title =
    find_tag_value(tags, KEYS_ALBUM).map(|s| s.to_string()).unwrap_or_else(|| "Unknown Album".to_string());

  let date_str = find_tag_value(tags, KEYS_DATE).map(|s| s.to_string());
  let raw_genre = find_tag_value(tags, KEYS_GENRE).map(|s| s.to_string());

  let (genres, styles) = parse_genre_and_style(raw_genre)?;

  Ok(Release {
    id: ReleaseId::new(),
    title: album_title,
    release_type: vec![ReleaseType::Album], // Heurística inicial; ajustar si se detectan EP / Single.
    main_artist_ids: Vec::new(),
    release_tracks: Vec::new(),
    release_date: date_str,
    artworks: Vec::new(),
    genres,
    styles,
  })
}

fn parse_genre_and_style(raw: Option<String>) -> Result<(Vec<Genre>, Vec<Style>), MetadataError> {
  let Some(source) = raw else {
    return Ok((Vec::new(), Vec::new()));
  };

  // Se permite que falle tanto Genre como Style sin abortar el análisis completo.
  if let Ok(genre) = Genre::from_str(&source) {
    Ok((vec![genre], Vec::new()))
  } else {
    let style = Style::from_str(&source).unwrap();
    Ok((Vec::new(), vec![style]))
  }
}

fn build_release_track(
  song: &Song,
  release: &Release,
  tags: &HashMap<String, String>,
  audio_details: AudioDetails,
  file_details: FileDetails,
) -> ReleaseTrack {
  let track_number = find_tag_number(tags, KEYS_TRACK_NUMBER).unwrap_or(1);
  let disc_number = find_tag_number(tags, KEYS_DISC_NUMBER).unwrap_or(1);

  ReleaseTrack {
    id: ReleaseTrackId::new(),
    song_id: song.id,
    release_id: release.id,
    track_number,
    disc_number,
    title_override: None,
    artist_credits: Vec::new(),
    audio_details,
    file_details,
  }
}

// ----- extracción de propiedades de audio ------------

fn extract_container_level_audio_info(context: &ffmpeg::format::context::Input) -> (Duration, Option<u32>) {
  let duration_micros = context.duration();
  let duration = if duration_micros > 0 { Duration::from_micros(duration_micros as u64) } else { Duration::ZERO };

  let bitrate_kbps = if context.bit_rate() > 0 { Some((context.bit_rate() / 1000) as u32) } else { None };

  (duration, bitrate_kbps)
}

fn extract_stream_level_audio_info(context: &mut ffmpeg::format::context::Input) -> (Option<u32>, Option<u8>) {
  let audio_stream = context.streams().best(ffmpeg::media::Type::Audio);

  if let Some(stream) = audio_stream {
    let params = stream.parameters();
    if let Ok(ctx) = ffmpeg::codec::context::Context::from_parameters(params) {
      if let Ok(audio_decoder) = ctx.decoder().audio() {
        let rate = audio_decoder.rate();
        let channels = audio_decoder.channels();
        return (Some(rate), Some(channels as u8));
      }
    }
  }

  (None, None)
}

fn run_spectral_analysis(
  path: &Path,
  analysis_config: Option<AnalysisConfig>,
) -> Result<Option<AudioQuality>, MetadataError> {
  let Some(config) = analysis_config else {
    return Ok(None);
  };

  let mut analyzer = SpectralAnalyzer::new(config);
  match analyzer.analyze_file(path) {
    Ok(result) => Ok(Some(result)),
    Err(e) => {
      // No queremos que un fallo de análisis cancele la extracción de metadatos.
      eprintln!("Aviso: fallo en análisis espectral para {:?}: {e}", path);
      Ok(None)
    }
  }
}
