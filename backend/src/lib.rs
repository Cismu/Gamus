use gamus_core::ports::LibraryRepository;
use gamus_storage::SqliteLibraryRepository;
use serde::Serialize;

#[derive(Serialize)]
struct ArtistDto {
  id: String,
  name: String,
}

#[tauri::command]
fn list_artists() -> Result<Vec<ArtistDto>, String> {
  let repo =
    SqliteLibraryRepository::new("crates/gamus-storage/gamus.db").map_err(|e| e.to_string())?;

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
  let repo =
    SqliteLibraryRepository::new("crates/gamus-storage/gamus.db").map_err(|e| e.to_string())?;

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

  let repo =
    SqliteLibraryRepository::new("crates/gamus-storage/gamus.db").map_err(|e| e.to_string())?;

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

  let repo = SqliteLibraryRepository::new("gamus.db").map_err(|e| e.to_string())?;

  let song = Song { id: SongId::new(), title: input.title, acoustid: input.acoustid };

  repo.save_song(&song).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![list_artists, list_songs, create_artist, create_song])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
