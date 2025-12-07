use futures::StreamExt;
use gamus_fs::async_walker::{Filtering, WalkConfig, walk_filtered};
use std::time::Instant;

#[tokio::main]
async fn main() {
  let start_time = Instant::now();

  let cfg = WalkConfig { follow_symlinks: false, max_depth: 50, dedup_dirs: true };
  let root = "/home/undead34/Music";

  let entries = walk_filtered(root, cfg, |entry| {
    let path = entry.path.clone();
    async move {
      if let Some(name) = path.file_name() {
        if name.to_string_lossy().starts_with('.') {
          return Filtering::IgnoreDir;
        }
      }
      if path.extension().map_or(false, |e| e == "tmp") {
        return Filtering::Ignore;
      }
      Filtering::Continue
    }
  });

  tokio::pin!(entries);

  let mut count = 0;

  while let Some(res) = entries.next().await {
    match res {
      Ok(_) => {
        count += 1;
      }
      Err(_) => (),
    }
  }

  // 3. Detenemos el cronómetro y calculamos la duración
  let duration = start_time.elapsed();

  println!("------------------------------------------------");
  println!("Procesamiento completado.");
  println!("Archivos encontrados: {}", count);
  println!("Tiempo de ejecución: {:.2?}", duration); // Formato legible
  println!("------------------------------------------------");
}
