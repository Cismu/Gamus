use gamus_core::domain::artist::Artist;
use gamus_core::domain::ids::ArtistId;
use gamus_core::ports::LibraryRepository;
use gamus_storage::SqliteLibraryRepository;
use uuid::Uuid;

fn main() {
  // ajusta la ruta si tu DATABASE_URL es otra
  let repo = SqliteLibraryRepository::new("gamus.db").expect("failed to connect");

  let artist = Artist {
    id: ArtistId::from_uuid(Uuid::new_v4()),
    name: "Test Artist".to_string(),
    variations: vec!["TA".to_string()],
    bio: Some("Artista de prueba de Gamus".to_string()),
    sites: vec!["https://example.com".to_string()],
  };

  println!("Saving artist with id = {}", artist.id);

  repo.save_artist(&artist).expect("failed to save artist");

  let loaded = repo.find_artist(artist.id).expect("failed to load artist");

  println!("Loaded from DB: {loaded:?}");
}
