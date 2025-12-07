use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn atomic_write_str(path: &Path, contents: &str) -> io::Result<()> {
  let tmp_path = path.with_extension("tmp");

  {
    let mut tmp_file = fs::File::create(&tmp_path)?;
    tmp_file.write_all(contents.as_bytes())?;
    tmp_file.sync_all()?;
  }

  fs::rename(&tmp_path, path)?;
  Ok(())
}
