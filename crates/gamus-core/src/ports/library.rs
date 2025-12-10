use crate::domain::ids::{ArtistId, ReleaseId, SongId};
use crate::domain::{artist::Artist, release::Release, song::Song};
use crate::errors::CoreError;

pub trait Library {
  // --- Métodos de Comando (Escritura) ---
  fn save_artist(&self, artist: &Artist) -> Result<(), CoreError>;
  fn save_song(&self, song: &Song) -> Result<(), CoreError>;
  fn save_release(&self, release: &Release) -> Result<(), CoreError>;

  // --- Métodos de Consulta (Lectura) por ID ---
  fn find_artist(&self, id: ArtistId) -> Result<Option<Artist>, CoreError>;
  fn find_song(&self, id: SongId) -> Result<Option<Song>, CoreError>;
  fn find_release(&self, id: ReleaseId) -> Result<Option<Release>, CoreError>;

  // --- Métodos de Consulta (Lectura) de Listado ---
  fn list_artists(&self) -> Result<Vec<Artist>, CoreError>;
  fn list_songs(&self) -> Result<Vec<Song>, CoreError>;
  fn list_releases(&self) -> Result<Vec<Release>, CoreError>;
}
