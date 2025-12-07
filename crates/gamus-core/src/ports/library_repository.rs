use crate::domain::ids::{ArtistId, ReleaseId, SongId};
use crate::domain::{artist::Artist, release::Release, song::Song};
use crate::errors::CoreError;

pub trait LibraryRepository {
  fn save_artist(&self, artist: &Artist) -> Result<(), CoreError>;
  fn save_song(&self, song: &Song) -> Result<(), CoreError>;
  fn save_release(&self, release: &Release) -> Result<(), CoreError>;

  fn find_artist(&self, id: ArtistId) -> Result<Option<Artist>, CoreError>;
  fn find_song(&self, id: SongId) -> Result<Option<Song>, CoreError>;
  fn find_release(&self, id: ReleaseId) -> Result<Option<Release>, CoreError>;
}
