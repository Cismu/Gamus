use gamus_core::domain::{SongId, song::Song};
use gamus_core::ports::LibraryRepository;
use gamus_storage::SqliteLibraryRepository;

fn main() {
  // Usa la misma ruta que en smoke_artist
  let repo = SqliteLibraryRepository::new("gamus.db").expect("failed to connect");

  let song = Song {
    id: SongId::new(),
    title: "Test Song".to_string(),
    acoustid: Some("dummy-acoustid-123".to_string()),
  };

  println!("Saving song with id = {}", song.id);

  repo.save_song(&song).expect("failed to save song");

  let loaded = repo.find_song(song.id).expect("failed to load song");

  println!("Loaded from DB: {loaded:?}");
}
