use serde::{Deserialize, Serialize};
use std::fmt;

/// Calificación promedio de un ítem (canción, release, etc.).
///
/// Distingue explícitamente entre:
/// - [`AvgRating::Unrated`]: el usuario nunca ha puntuado este ítem.
/// - [`AvgRating::Rated`]: existe al menos una valoración registrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AvgRating {
  /// El ítem no tiene valoraciones asociadas.
  Unrated,
  /// Calificación promedio basada en una o más valoraciones.
  Rated(Rating),
}

impl Default for AvgRating {
  fn default() -> Self {
    AvgRating::Unrated
  }
}

impl fmt::Display for AvgRating {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AvgRating::Unrated => write!(f, "☆☆☆☆☆"),
      AvgRating::Rated(rating) => fmt::Display::fmt(rating, f),
    }
  }
}

/// Representa una valoración en una escala de 0.0 a 5.0 con precisión fija.
///
/// Internamente se guarda como un entero (`u32`) en formato *fixed-point*
/// con 4 decimales de precisión. Es decir:
///
/// - `0.0`  → `0`
/// - `3.5`  → `35000`
/// - `5.0`  → `50000`
///
/// Esto evita errores de redondeo típicos de los `f32` al acumular operaciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rating(u32);

impl Rating {
  /// Factor de escala usado para representar decimales (4 cifras).
  const SCALE_FACTOR: u32 = 10_000;
  /// Valor máximo permitido: 5.0 escalado.
  const MAX_VALUE: u32 = 5 * Self::SCALE_FACTOR;

  /// Crea una nueva `Rating` a partir de un valor en coma flotante.
  ///
  /// El valor debe estar en el rango `[0.0, 5.0]` (inclusive). Si está fuera
  /// de rango, la función devuelve `None`.
  pub fn new(value: f32) -> Option<Self> {
    if !(0.0..=5.0).contains(&value) {
      return None;
    }

    let scaled_value = (value * Self::SCALE_FACTOR as f32).round() as u32;

    if scaled_value > Self::MAX_VALUE {
      return None;
    }

    Some(Self(scaled_value))
  }

  /// Devuelve la valoración como `f32` en la escala `[0.0, 5.0]`.
  pub fn as_f32(&self) -> f32 {
    self.0 as f32 / Self::SCALE_FACTOR as f32
  }
}

impl fmt::Display for Rating {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Nota de diseño:
    // Usamos `floor` en vez de `round` para que:
    // - 4.1 → ★★★★☆
    // - 4.9 → ★★★★☆
    // y solo 5.0 llegue a ★★★★★.
    let full_stars = self.as_f32().floor() as usize;
    let empty_stars = 5 - full_stars;

    for _ in 0..full_stars {
      write!(f, "★")?;
    }
    for _ in 0..empty_stars {
      write!(f, "☆")?;
    }

    Ok(())
  }
}
