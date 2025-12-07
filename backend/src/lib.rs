use gamus_storage::SqliteLibraryRepository;
use serde::Serialize;

#[derive(Serialize)]
struct ArtistDto {
  id: String,
  name: String,
}

#[tauri::command]
fn list_artists() -> Result<Vec<ArtistDto>, String> {
  // Usa la misma ruta que usaste en el smoke test
  let repo =
    SqliteLibraryRepository::new("../crates/gamus-storage/gamus.db").map_err(|e| e.to_string())?;

  let artists = repo.list_artists().map_err(|e| e.to_string())?;

  let dtos =
    artists.into_iter().map(|a| ArtistDto { id: a.id.to_string(), name: a.name }).collect();

  Ok(dtos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![list_artists])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
