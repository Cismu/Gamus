use crate::domain::ids::SongId;
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
}
