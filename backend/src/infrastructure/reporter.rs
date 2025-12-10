use async_trait::async_trait;
use gamus_core::ports::ProgressReporter;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// DTO for serializing error details to the frontend.
#[derive(Clone, Serialize)]
struct ErrorPayload {
  path: String,
  error: String,
}

/// A `ProgressReporter` implementation that bridges backend events to the Tauri frontend.
///
/// This struct holds a reference to the `AppHandle`, allowing it to emit global events
#[derive(Clone)]
pub struct TauriReporter {
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
    // Fire-and-forget: We ignore emission errors (e.g., if the webview is closed)
    // to prevent UI state from crashing the backend process.
    let _ = self.app_handle.emit("library:import:start", total_files);
  }

  async fn on_success(&self, path: &str) {
    let _ = self.app_handle.emit("library:import:success", path);
  }

  async fn on_error(&self, path: &str, error: &str) {
    let payload = ErrorPayload { path: path.to_string(), error: error.to_string() };
    let _ = self.app_handle.emit("library:import:error", payload);
  }

  async fn finish(&self) {
    let _ = self.app_handle.emit("library:import:finish", ());
  }
}
