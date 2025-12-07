use gamus_fs::scan_music_files;

#[tokio::main]
async fn main() {
  let root = "/home/undead34/Music";

  match scan_music_files(root).await {
    Ok(files) => {
      println!("Encontrados {} archivos de audio:", files.len());
      for f in files {
        println!("{} | {} bytes | modified={}s", f.path.display(), f.size, f.modified,);
      }
    }
    Err(e) => eprintln!("Error al escanear: {e}"),
  }
}
