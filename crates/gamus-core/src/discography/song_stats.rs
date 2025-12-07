use serde::{Deserialize, Serialize};

use crate::discography::rating::AvgRating;

/// Estadísticas de interacción del usuario con la canción.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SongStats {
  pub avg_rating: AvgRating,
  pub ratings: u32,
  pub comments: Vec<String>,
}
