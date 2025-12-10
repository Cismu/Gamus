use std::io::Read;
use std::path::Path;

/// Identifies the physical device ID for a given file path.
///
/// On Unix-like systems, this relies on the `st_dev` field from file metadata,
/// which uniquely identifies the mounted filesystem/partition.
#[cfg(unix)]
pub fn device_id(path: &Path) -> Result<String, std::io::Error> {
  use std::os::unix::fs::MetadataExt;
  // Security Note: Ensure the user has read permissions on the file path,
  // otherwise `metadata` will return a permission denied error.
  let meta = std::fs::metadata(path)?;
  Ok(meta.dev().to_string())
}

/// Identifies the physical device ID for a given file path (Windows implementation).
///
/// On Windows, this implementation extracts the drive letter (e.g., "C:") from the path prefix.
/// Note: This is a heuristic approximation. It groups partitions correctly but does not
/// distinguish between multiple partitions on the same physical disk, which is acceptable
/// for basic throttling but less precise than the Unix `st_dev`.
#[cfg(windows)]
pub fn device_id(path: &Path) -> Result<String, std::io::Error> {
  use std::path::Component;
  let drive = match path.components().next() {
    Some(Component::Prefix(prefix)) => match prefix.kind() {
      // Handle both standard "C:" and verbatim "\\?\C:" prefixes.
      std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
        format!("{}:", letter as char)
      }
      _ => "OTHER_DRIVE".into(),
    },
    _ => "NO_DRIVE".into(),
  };
  Ok(drive)
}

/// Fallback implementation for unsupported platforms.
/// Returns a constant ID, effectively disabling device-aware parallelization features.
#[cfg(not(any(unix, windows)))]
pub fn device_id(_path: &Path) -> Result<String, std::io::Error> {
  Ok("UNKNOWN_DEVICE".into())
}

/// Performs a blocking micro-benchmark to estimate read throughput.
///
/// Reads `sample_bytes` from the beginning of `sample_path` to calculate MB/s.
///
/// # Performance Considerations
/// * **Blocking:** This function blocks the thread. Do not call this directly from an async executor.
/// * **Caching:** The OS page cache may skew results if the file was recently accessed.
///   For the purpose of this application (ingestion throttling), cached speeds are an acceptable
///   upper bound estimate.
pub fn measure_device_throughput(sample_path: &Path, sample_bytes: usize) -> Result<f64, std::io::Error> {
  let start = std::time::Instant::now();
  let mut file = std::fs::File::open(sample_path)?;

  // Allocate buffer on heap to avoid stack overflow for large sample sizes.
  let mut buf = vec![0u8; sample_bytes];
  let mut read_total = 0usize;

  // Read loop ensures we fill the buffer even if the OS interrupts the read call.
  while read_total < sample_bytes {
    let n = file.read(&mut buf[read_total..])?;
    if n == 0 {
      break; // EOF reached before sample size
    }
    read_total += n;
  }

  let secs = start.elapsed().as_secs_f64();
  if secs == 0.0 {
    // Prevent division by zero if the read is instant (e.g., very small sample or fully cached).
    Ok(0.0)
  } else {
    Ok((read_total as f64) / 1_048_576.0 / secs) // Convert bytes to MB/s
  }
}
