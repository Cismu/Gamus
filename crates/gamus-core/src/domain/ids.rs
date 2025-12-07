use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtistId(Uuid);

impl ArtistId {
  /// Genera un nuevo identificador único.
  pub fn new() -> Self {
    ArtistId(Uuid::new_v4())
  }

  /// Construye un `ArtistId` a partir de un `Uuid` existente.
  pub fn from_uuid(u: Uuid) -> Self {
    ArtistId(u)
  }

  /// Devuelve el `Uuid` interno.
  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

impl From<Uuid> for ArtistId {
  fn from(u: Uuid) -> Self {
    ArtistId(u)
  }
}

impl From<ArtistId> for Uuid {
  fn from(id: ArtistId) -> Self {
    id.0
  }
}

impl fmt::Display for ArtistId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SongId(Uuid);

impl SongId {
  pub fn new() -> Self {
    SongId(Uuid::new_v4())
  }

  pub fn from_uuid(u: Uuid) -> Self {
    SongId(u)
  }

  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

impl From<Uuid> for SongId {
  fn from(u: Uuid) -> Self {
    SongId(u)
  }
}

impl From<SongId> for Uuid {
  fn from(id: SongId) -> Self {
    id.0
  }
}

impl fmt::Display for SongId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// Identificador único para un lanzamiento (`Release`).
///
/// Este ID es completamente abstracto y no depende de ninguna fuente externa.
/// Se genera con UUID v4 para garantizar unicidad global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseId(Uuid);

impl ReleaseId {
  /// Crea un nuevo `ReleaseId` único.
  pub fn new() -> Self {
    ReleaseId(Uuid::new_v4())
  }

  /// Construye un `ReleaseId` desde un UUID ya existente.
  pub fn from_uuid(u: Uuid) -> Self {
    ReleaseId(u)
  }

  /// Devuelve el valor UUID interno.
  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

impl From<Uuid> for ReleaseId {
  fn from(u: Uuid) -> Self {
    ReleaseId(u)
  }
}

impl From<ReleaseId> for Uuid {
  fn from(id: ReleaseId) -> Self {
    id.0
  }
}

impl fmt::Display for ReleaseId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// Identificador único y global para una pista concreta dentro de un release.
///
/// A diferencia de `SongId` (obra abstracta) o `ReleaseId` (producto),
/// este ID identifica *la instancia física* (archivo + metadatos) de una canción
/// dentro de un lanzamiento.
///
/// Ejemplo:
/// - Track 3 del CD1 del álbum
/// - Track 7 de la compilación japonesa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseTrackId(Uuid);

impl ReleaseTrackId {
  /// Genera un nuevo ID único.
  pub fn new() -> Self {
    ReleaseTrackId(Uuid::new_v4())
  }

  /// Crea el ID desde un UUID existente.
  pub fn from_uuid(uuid: Uuid) -> Self {
    ReleaseTrackId(uuid)
  }

  /// Accede al UUID interno.
  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

impl From<Uuid> for ReleaseTrackId {
  fn from(u: Uuid) -> Self {
    ReleaseTrackId(u)
  }
}

impl From<ReleaseTrackId> for Uuid {
  fn from(id: ReleaseTrackId) -> Self {
    id.0
  }
}

impl fmt::Display for ReleaseTrackId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}
