use async_trait::async_trait;

// Este es el nuevo puerto.
// El frontend (Tauri) implementará esto para actualizar la UI.
#[async_trait]
pub trait ProgressReporter: Send + Sync + Clone {
  async fn start(&self, total_files: usize);
  async fn on_success(&self, path: &str); // O pasar un objeto 'ImportResult'
  async fn on_error(&self, path: &str, error: &str);
  async fn finish(&self);
}
