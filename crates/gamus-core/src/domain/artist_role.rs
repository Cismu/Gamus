use crate::domain::ids::{ArtistId, ReleaseTrackId};
use serde::{Deserialize, Serialize};

/// Rol específico de un artista respecto a una pista concreta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtistRole {
  /// Artista principal que interpreta la pista.
  Performer,
  /// Artista invitado.
  Featured,
  /// Compositor (música / letra).
  Composer,
  /// Productor musical.
  Producer,
  /// Remixer de una pista existente.
  Remixer,
}

/// Crédito de un artista en una pista concreta de un release.
///
/// Esto representa la misma idea que `release_track_artists` en la base de datos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTrackArtistCredit {
  pub release_track_id: ReleaseTrackId,
  pub artist_id: ArtistId,
  pub role: ArtistRole,
  // opcional: orden en los créditos
  pub position: Option<u32>,
}
