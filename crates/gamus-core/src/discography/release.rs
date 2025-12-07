use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::genre_styles::{Genre, Style};
use crate::discography::ids::{ArtistId, ReleaseId, ReleaseTrackId};
use crate::discography::release_type::ReleaseType;

/// Representa un lanzamiento musical.
///
/// Un *Release* agrupa un conjunto de pistas (`ReleaseTrack`) y define
/// información editorial como:
/// - título,
/// - formato (álbum, EP, etc.),
/// - artistas principales,
/// - fecha de lanzamiento,
/// - estilos y géneros.
///
/// Semánticamente, corresponde al concepto de "objeto publicado", no a la canción individual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
  /// Identificador único del lanzamiento.
  pub id: ReleaseId,

  /// Título del lanzamiento tal como aparece oficialmente.
  pub title: String,

  /// Lista de tipos que describen el release (Album, EP, Mix, Custom…)
  /// Se usa `Vec` porque algunos sistemas clasifican un mismo launch como *Album + Compilation*, por ejemplo.
  pub release_type: Vec<ReleaseType>,

  /// IDs de los artistas principales asociados.
  pub main_artist_ids: Vec<ArtistId>,

  /// IDs de las pistas que pertenecen a este release.
  pub release_tracks: Vec<ReleaseTrackId>,

  /// Fecha oficial de publicación del lanzamiento.
  ///
  /// [todo]: temporalmente se usa `String` porque los metadatos musicales pueden venir
  /// en formatos ambiguos ("1998", "1998-05", "May 1998", etc.).  
  /// Es común procesarla luego hacia un tipo más estricto.
  pub release_date: Option<String>,

  /// Lista de artworks asociados (portadas, inserts, edición alternativa…)
  pub artworks: Vec<Artwork>,

  /// Géneros asignados al lanzamiento.
  pub genres: Vec<Genre>,

  /// Estilos específicos (más granulares que los géneros).
  pub styles: Vec<Style>,
}

/// Representa una imagen asociada al release
/// (por ejemplo: portada, contraportada, ediciones alternativas).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artwork {
  /// Ruta local del archivo de imagen.
  pub path: PathBuf,

  /// MIME type (por ejemplo, `"image/jpeg"`).
  pub mime_type: String,

  /// Descripción opcional (e.g., `"Portada japonesa"`, `"Edición limitada"`).
  pub description: Option<String>,

  /// Hash del contenido de la imagen, usado para identificar duplicados.
  pub hash: String,

  /// Créditos opcionales del artwork (fotógrafo, diseñador, etc.).
  pub credits: Option<String>,
}
