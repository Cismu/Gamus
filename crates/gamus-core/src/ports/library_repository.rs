use crate::domain::{ArtistId, ReleaseId, SongId};
use crate::domain::{artist::Artist, release::Release, song::Song};

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
  #[error("entity not found")]
  NotFound,
  #[error("storage error: {0}")]
  Storage(String),
  // luego puedes expandir mejor
}

pub trait LibraryRepository {
  fn save_song(&self, song: &Song) -> Result<(), RepoError>;
  fn save_artist(&self, artist: &Artist) -> Result<(), RepoError>;
  fn save_release(&self, release: &Release) -> Result<(), RepoError>;

  fn find_song(&self, id: SongId) -> Result<Option<Song>, RepoError>;
  fn find_artist(&self, id: ArtistId) -> Result<Option<Artist>, RepoError>;
  fn find_release(&self, id: ReleaseId) -> Result<Option<Release>, RepoError>;
}
