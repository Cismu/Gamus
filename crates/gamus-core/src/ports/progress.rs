use async_trait::async_trait;

/// Contract for reporting the status of long-running operations.
///
/// Designed to decouple the core logic (ingestion/scanning) from the UI or logging mechanism.
///
/// # Concurrency
/// Implementations must be `Send + Sync + Clone` to facilitate sharing across
/// asynchronous task boundaries (e.g., worker threads processing different scan groups).
#[async_trait]
pub trait ProgressReporter: Send + Sync + Clone {
  /// Signals the beginning of a batch operation.
  async fn start(&self, total_files: usize);

  /// Reports a single successful unit of work.
  async fn on_success(&self, path: &str);

  /// Reports a failure for a specific unit of work without aborting the batch.
  async fn on_error(&self, path: &str, error: &str);

  /// Signals that the batch operation has concluded (successfully or otherwise).
  async fn finish(&self);
}
