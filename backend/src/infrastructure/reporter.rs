use async_trait::async_trait;
use gamus_core::ports::ProgressReporter;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

// Estructuras para los payloads de los eventos (para que el JSON sea limpio)
#[derive(Clone, Serialize)]
struct ErrorPayload {
  path: String,
  error: String,
}

#[derive(Clone)]
pub struct TauriReporter {
  // AppHandle es barato de clonar y Thread-Safe
  app_handle: AppHandle,
}

impl TauriReporter {
  pub fn new(app_handle: AppHandle) -> Self {
    Self { app_handle }
  }
}

#[async_trait]
impl ProgressReporter for TauriReporter {
  async fn start(&self, total_files: usize) {
    // Evento: "library:import:start" -> Payload: número (total)
    let _ = self.app_handle.emit("library:import:start", total_files);
  }

  async fn on_success(&self, path: &str) {
    // Evento: "library:import:success" -> Payload: string (path)
    let _ = self.app_handle.emit("library:import:success", path);
  }

  async fn on_error(&self, path: &str, error: &str) {
    // Evento: "library:import:error" -> Payload: objeto { path, error }
    let payload = ErrorPayload { path: path.to_string(), error: error.to_string() };
    let _ = self.app_handle.emit("library:import:error", payload);
  }

  async fn finish(&self) {
    // Evento: "library:import:finish" -> Payload: null
    let _ = self.app_handle.emit("library:import:finish", ());
  }
}
