use std::io::Read;
use std::path::Path;

#[cfg(unix)]
pub fn device_id(path: &Path) -> Result<String, std::io::Error> {
  use std::os::unix::fs::MetadataExt;
  let meta = std::fs::metadata(path)?;
  Ok(meta.dev().to_string())
}

#[cfg(windows)]
pub fn device_id(path: &Path) -> Result<String, std::io::Error> {
  use std::path::Component;
  let drive = match path.components().next() {
    Some(Component::Prefix(prefix)) => match prefix.kind() {
      std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
        format!("{}:", letter as char)
      }
      _ => "OTHER_DRIVE".into(),
    },
    _ => "NO_DRIVE".into(),
  };
  Ok(drive)
}

#[cfg(not(any(unix, windows)))]
pub fn device_id(_path: &Path) -> Result<String, std::io::Error> {
  Ok("UNKNOWN_DEVICE".into())
}

/// Lee `sample_bytes` del principio del archivo y devuelve MB/s.
pub fn measure_device_throughput(
  sample_path: &Path,
  sample_bytes: usize,
) -> Result<f64, std::io::Error> {
  let start = std::time::Instant::now();
  let mut file = std::fs::File::open(sample_path)?;
  let mut buf = vec![0u8; sample_bytes];
  let mut read_total = 0usize;

  while read_total < sample_bytes {
    let n = file.read(&mut buf[read_total..])?;
    if n == 0 {
      break; // EOF
    }
    read_total += n;
  }

  let secs = start.elapsed().as_secs_f64();
  if secs == 0.0 {
    Ok(0.0)
  } else {
    Ok((read_total as f64) / 1_048_576.0 / secs) // MB/s
  }
}
