use gamus_core::ports::{FileScanner, LibraryRepository};
use gamus_storage::SqliteLibraryRepository;

use gamus_scanner::GamusFileScanner;
use gamus_scanner::config::ScannerConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize)]
struct ArtistDto {
  id: String,
  name: String,
}

#[tauri::command]
fn list_artists() -> Result<Vec<ArtistDto>, String> {
  let repo = SqliteLibraryRepository::new_from_config().map_err(|e| e.to_string())?;

  let artists = repo.list_artists().map_err(|e| e.to_string())?;

  let dtos =
    artists.into_iter().map(|a| ArtistDto { id: a.id.to_string(), name: a.name }).collect();

  Ok(dtos)
}

#[derive(serde::Serialize)]
struct SongDto {
  id: String,
  title: String,
}

#[tauri::command]
fn list_songs() -> Result<Vec<SongDto>, String> {
  let repo = SqliteLibraryRepository::new_from_config().map_err(|e| e.to_string())?;

  let songs = repo.list_songs().map_err(|e| e.to_string())?;

  Ok(songs.into_iter().map(|s| SongDto { id: s.id.to_string(), title: s.title }).collect())
}

#[derive(serde::Deserialize)]
struct CreateArtistInput {
  name: String,
  bio: Option<String>,
}

#[tauri::command]
fn create_artist(input: CreateArtistInput) -> Result<(), String> {
  use gamus_core::domain::artist::Artist;
  use gamus_core::domain::ids::ArtistId;

  let repo = SqliteLibraryRepository::new_from_config().map_err(|e| e.to_string())?;

  let artist = Artist {
    id: ArtistId::new(),
    name: input.name,
    variations: vec![],
    bio: input.bio,
    sites: vec![],
  };

  repo.save_artist(&artist).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
struct CreateSongInput {
  title: String,
  acoustid: Option<String>,
}

#[tauri::command]
fn create_song(input: CreateSongInput) -> Result<(), String> {
  use gamus_core::domain::ids::SongId;
  use gamus_core::domain::song::Song;

  let repo = SqliteLibraryRepository::new_from_config().map_err(|e| e.to_string())?;

  let song = Song { id: SongId::new(), title: input.title, acoustid: input.acoustid };

  repo.save_song(&song).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct DeviceSummaryDto {
  id: String,
  bandwidth_mb_s: Option<u64>,
  file_count: usize,
}

#[derive(Serialize)]
struct ScanSummaryDto {
  total_files: usize,
  devices: Vec<DeviceSummaryDto>,
}

#[tauri::command]
fn scan_library() -> Result<ScanSummaryDto, String> {
  let scanner = GamusFileScanner::new();

  let groups = scanner.scan_library_files().map_err(|e| e.to_string())?;

  let mut total_files = 0usize;
  let mut devices = Vec::new();

  for g in groups {
    let count = g.files.len();
    total_files += count;

    devices.push(DeviceSummaryDto {
      id: g.device.id,
      bandwidth_mb_s: g.device.bandwidth_mb_s,
      file_count: count,
    });
  }

  Ok(ScanSummaryDto { total_files, devices })
}

#[derive(Debug, Serialize, Deserialize)]
struct ScannerConfigDto {
  roots: Vec<String>,
  audio_exts: Vec<String>,
  ignore_hidden: bool,
  max_depth: Option<u32>,
}

impl From<ScannerConfig> for ScannerConfigDto {
  fn from(cfg: ScannerConfig) -> Self {
    ScannerConfigDto {
      roots: cfg.roots.into_iter().map(|p| p.to_string_lossy().to_string()).collect(),
      audio_exts: cfg.audio_exts,
      ignore_hidden: cfg.ignore_hidden,
      max_depth: cfg.max_depth,
    }
  }
}

impl From<ScannerConfigDto> for ScannerConfig {
  fn from(dto: ScannerConfigDto) -> Self {
    ScannerConfig {
      roots: dto.roots.into_iter().map(PathBuf::from).collect(),
      audio_exts: dto.audio_exts,
      ignore_hidden: dto.ignore_hidden,
      max_depth: dto.max_depth,
    }
  }
}

#[tauri::command]
fn get_scanner_config() -> Result<ScannerConfigDto, String> {
  let cfg = ScannerConfig::load().map_err(|e| e.to_string())?;
  Ok(ScannerConfigDto::from(cfg))
}

#[tauri::command]
fn save_scanner_config(input: ScannerConfigDto) -> Result<(), String> {
  let cfg = ScannerConfig::from(input);
  cfg.save().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      list_artists,
      list_songs,
      create_artist,
      create_song,
      scan_library,
      get_scanner_config,
      save_scanner_config,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
