mod config;
mod infrastructure;

use gamus_core::services::LibraryService;
use gamus_metadata::FfmpegMetadataExtractor;
use gamus_scanner::{GamusFileScanner, ScannerConfig};
use gamus_storage::SqliteLibraryRepository;

use tauri::{Manager, State};

use crate::config::ScannerConfigDto;
use infrastructure::system::gpu_tweak;
// Importamos el nuevo reporter
use infrastructure::reporter::TauriReporter;

// 1. Alias para no volvernos locos con los genéricos
type ConcreteLibraryService =
  LibraryService<GamusFileScanner, FfmpegMetadataExtractor, SqliteLibraryRepository, TauriReporter>;

struct AppState {
  library: ConcreteLibraryService,
}

#[tauri::command]
async fn library_import_full(state: State<'_, AppState>) -> Result<(), String> {
  // Al ser async, Tauri lo ejecuta en un hilo aparte automáticamente.
  // El servicio reportará progreso vía eventos mientras esto espera.
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
  gpu_tweak::apply_linux_patches();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      // 2. Inicializamos los puertos
      let storage = SqliteLibraryRepository::new_from_config()?;
      // Recuerda que ahora GamusFileScanner tiene caché interna
      let scanner = GamusFileScanner::new();
      let metadata = FfmpegMetadataExtractor::new();

      // 3. Inicializamos el Reporter con el AppHandle de Tauri
      // app.handle() nos da el control global para emitir eventos
      let reporter = TauriReporter::new(app.handle().clone());

      // 4. Inyectamos todo en el Servicio
      let service = LibraryService::new(scanner, metadata, storage, reporter);

      // 5. Gestionamos el Estado
      app.manage(AppState { library: service });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      library_import_full,
      scanner_get_config,
      scanner_save_config,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
