use gamus_scanner::GamusFileScanner as GamusScanner;
use gamus_storage::SqliteLibraryRepository as LibraryStorage;

use tauri::Manager;

struct AppState {
  scanner: GamusScanner,
  storage: LibraryStorage,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      let storage = LibraryStorage::new_from_config()?;

      app.manage(AppState { scanner: GamusScanner::new(), storage });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
