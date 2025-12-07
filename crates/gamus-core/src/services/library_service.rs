// crates/gamus-core/src/services/library_service.rs
use crate::domain::ids::SongId;
use crate::domain::{
  rating::{AvgRating, Rating},
  song::Song,
};
use crate::errors::CoreError;
use crate::ports::library_repository::LibraryRepository;

pub struct LibraryService<R>
where
  R: LibraryRepository,
{
  repo: R,
}

impl<R> LibraryService<R>
where
  R: LibraryRepository,
{
  pub fn new(repo: R) -> Self {
    Self { repo }
  }

  /// Asigna una valoración a una canción.
  ///
  /// Nota: por ahora esto pisa la media directamente; en el futuro
  /// podrías guardar el histórico de ratings y recalcular.
  pub fn rate_song(&self, song_id: SongId, rating: Rating) -> Result<(), CoreError> {
    let song = self.repo.find_song(song_id)?.ok_or(CoreError::NotFound)?;

    // aquí más adelante podrías tener song.stats, etc.
    // por ahora sería algo tipo:
    // song.statistics.avg_rating = AvgRating::Rated(rating);

    self.repo.save_song(&song)?;
    Ok(())
  }
}
