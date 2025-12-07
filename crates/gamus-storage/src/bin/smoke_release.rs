use gamus_core::domain::ReleaseId;
use gamus_core::domain::release::Release;
use gamus_core::ports::LibraryRepository;
use gamus_storage::SqliteLibraryRepository;

fn main() {
  let repo = SqliteLibraryRepository::new("gamus.db").expect("failed to connect");

  let release = Release {
    id: ReleaseId::new(),
    title: "Test Release".to_string(),
    release_type: vec![],    // luego probamos con tipos
    main_artist_ids: vec![], // más adelante
    release_tracks: vec![],  // más adelante
    release_date: Some("2025-12-07".to_string()),
    artworks: vec![],
    genres: vec![],
    styles: vec![],
  };

  println!("Saving release with id = {}", release.id);

  repo.save_release(&release).expect("failed to save release");

  let loaded = repo.find_release(release.id).expect("failed to load release");

  println!("Loaded from DB: {loaded:?}");
}
