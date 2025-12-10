mod config;
mod infrastructure;

use gamus_core::services::LibraryService;
use gamus_metadata::FfmpegProbe;
use gamus_scanner::{FsScanner, ScannerConfig};
use gamus_storage::LibraryStore;

use tauri::{Manager, State};

use crate::config::ScannerConfigDto;
use infrastructure::reporter::TauriReporter;
use infrastructure::system::gpu_tweak;

/// Type alias to simplify the generic signature of the Service.
type ConcreteLibraryService = LibraryService<FsScanner, FfmpegProbe, LibraryStore, TauriReporter>;

/// Global application state managed by Tauri.
struct AppState {
  library: ConcreteLibraryService,
}

/// Command: Triggers the full library ingestion process.
///
/// This is an async command that keeps the frontend awaiting until completion.
/// Progress updates are sent via the injected `TauriReporter` (side-channel events),
/// not the return value of this promise.
#[tauri::command]
async fn library_import_full(state: State<'_, AppState>) -> Result<(), String> {
  state.library.import_full().await.map_err(|e| e.to_string())
}

/// Command: Retrieves the current scanner configuration.
///
/// Maps the domain configuration object to a DTO suitable for serialization to the frontend.
#[tauri::command]
fn scanner_get_config() -> Result<ScannerConfigDto, String> {
  let cfg = ScannerConfig::load().map_err(|e| e.to_string())?;
  Ok(ScannerConfigDto::from(cfg))
}

/// Command: Persists updated scanner configuration from the frontend.
#[tauri::command]
fn scanner_save_config(input: ScannerConfigDto) -> Result<(), String> {
  let cfg = ScannerConfig::from(input);
  cfg.save().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Linux-specific workarounds for WebKitGTK rendering glitches/crashes on specific GPUs.
  gpu_tweak::apply_linux_patches();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      // --- Dependency Injection Phase ---

      // 1. Persistence Adapter (SQLite)
      // Connects to the DB defined in config. May fail if filesystem is locked/invalid.
      let storage = LibraryStore::new_from_config()?;

      // 2. Scanner Adapter (Filesystem)
      // Maintains throughput cache state.
      let scanner = FsScanner::new();

      // 3. Metadata Adapter (FFmpeg)
      // Initializes internal FFmpeg contexts.
      let metadata = FfmpegProbe::default();

      // 4. Output Port Adapter (UI Events)
      // Wraps the Tauri AppHandle to emit events back to the WebView.
      let reporter = TauriReporter::new(app.handle().clone());

      // 5. Service Wiring
      // Inject all adapters into the core domain service.
      let library = LibraryService::new(scanner, metadata, storage, reporter);

      // 6. State Registration
      // Moves the service instance into Tauri's managed state container.
      app.manage(AppState { library });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![library_import_full, scanner_get_config, scanner_save_config,])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
