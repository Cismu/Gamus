use crate::discography::ids::ArtistId;
use serde::{Deserialize, Serialize};

/// Representa a un artista dentro del sistema.
///
/// Un artista es una entidad abstracta que agrupa todas sus obras,
/// contribuciones y variaciones de nombre. No representa un archivo
/// específico ni un rol concreto: es la identidad artística base.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artist {
  /// Identificador único del artista.
  pub id: ArtistId,

  /// Nombre principal (canónico) del artista.
  pub name: String,

  /// Variaciones conocidas del nombre (alias, traducciones, romanizaciones).
  pub variations: Vec<String>,

  /// Información biográfica opcional del artista.
  pub bio: Option<String>,

  /// Enlaces relevantes: páginas oficiales, redes, Wikipedia, Discogs, etc.
  pub sites: Vec<String>,
}
