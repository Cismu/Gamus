use crate::domain::ids::{ArtistId, SongId};
use serde::{Deserialize, Serialize};

/// La Canción (Song): La obra musical abstracta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Song {
  /// Identificador único de la canción dentro del sistema.
  pub id: SongId,
  /// La "huella digital" acústica de la canción, para verificación online.
  pub acoustid: Option<String>,
  /// El título de la canción.
  pub title: String,
  /// El/los intérprete(s) principal(es) de la canción.
  pub performer_ids: Vec<ArtistId>,
  /// El/los artista(s) invitado(s) o colaborador(es).
  pub featured_ids: Vec<ArtistId>,
  /// El/los compositor(es) de la letra y/o música.
  pub composer_ids: Vec<ArtistId>,
  /// El/los productor(es) que supervisaron la grabación.
  pub producer_ids: Vec<ArtistId>,
}
