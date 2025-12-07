use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Representa el tipo de lanzamiento.
///
/// Este enum sigue la clasificación clásica de la industria musical
/// (Album, EP, Single, etc.) pero también permite valores personalizados
/// mediante [`ReleaseType::Custom`].
///
/// Ejemplos típicos:
/// - `Album`
/// - `EP`
/// - `Single`
/// - `DJ-Mix`
/// - `Bootleg` (entraría como `Custom("Bootleg")`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseType {
  /// Un álbum completo.
  Album,
  /// Extended Play: más corto que un álbum, más largo que un single.
  EP,
  /// Un lanzamiento de una sola pista o pocas pistas.
  Single,
  /// Recopilación de pistas de varios releases o artistas.
  Compilation,
  /// Mezcla continua o set estilo DJ.
  Mix,
  /// Valor no estándar definido por el usuario.
  Custom(String),
}

impl FromStr for ReleaseType {
  type Err = std::convert::Infallible;

  /// Convierte una cadena en un `ReleaseType`.
  ///
  /// Las coincidencias conocidas se normalizan (minúsculas, trimming).
  /// Si no coincide con ninguno de los valores estándar, se retorna
  /// `ReleaseType::Custom(s.to_string())`.
  ///
  /// Esto significa que **parsear nunca falla**.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let normalized = s.trim().to_lowercase();

    let rt = match normalized.as_str() {
      "album" | "cd" | "lp" | "vinyl" | "album/cd" => ReleaseType::Album,
      "ep" => ReleaseType::EP,
      "single" => ReleaseType::Single,
      "compilation" => ReleaseType::Compilation,
      "mix" | "dj-mix" | "mixtape" => ReleaseType::Mix,
      _ => ReleaseType::Custom(s.to_string()),
    };

    Ok(rt)
  }
}

impl fmt::Display for ReleaseType {
  /// Devuelve un nombre legible del tipo de lanzamiento.
  ///
  /// Los tipos estándar se imprimen con formato bonito.
  /// `Custom` imprime directamente el valor proporcionado.
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ReleaseType::Album => write!(f, "Album"),
      ReleaseType::EP => write!(f, "EP"),
      ReleaseType::Single => write!(f, "Single"),
      ReleaseType::Compilation => write!(f, "Compilation"),
      ReleaseType::Mix => write!(f, "Mix"),
      ReleaseType::Custom(s) => write!(f, "{s}"),
    }
  }
}
