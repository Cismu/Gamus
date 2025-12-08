mod config;

use gamus_core::services::LibraryService;
use gamus_metadata::FfmpegMetadataExtractor;
use gamus_scanner::{GamusFileScanner, ScannerConfig};
use gamus_storage::SqliteLibraryRepository;

use tauri::{Manager, State};

use crate::config::ScannerConfigDto;

struct AppState {
  library: LibraryService<GamusFileScanner, FfmpegMetadataExtractor, SqliteLibraryRepository>,
}

#[tauri::command]
async fn library_import_full(state: State<'_, AppState>) -> Result<(), String> {
  state.library.import_full().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn scanner_get_config() -> Result<ScannerConfigDto, String> {
  let cfg = ScannerConfig::load().map_err(|e| e.to_string())?;
  Ok(ScannerConfigDto::from(cfg))
}

#[tauri::command]
fn scanner_save_config(input: ScannerConfigDto) -> Result<(), String> {
  let cfg = ScannerConfig::from(input);
  cfg.save().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      let storage = SqliteLibraryRepository::new_from_config()?;
      let scanner = GamusFileScanner::new();
      let metadata = FfmpegMetadataExtractor::new();

      let service = LibraryService::new(scanner, metadata, storage);

      app.manage(AppState { library: service });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      library_import_full,
      // Configs
      scanner_get_config,
      scanner_save_config,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
