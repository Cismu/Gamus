use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

use crate::domain::ids::{ReleaseId, ReleaseTrackId, SongId};

/// Identificador único y global para una pista concreta dentro de un release.
///
/// A diferencia de `SongId` (obra abstracta) o `ReleaseId` (producto),
/// este ID identifica *la instancia física* (archivo + metadatos) de una canción
/// dentro de un lanzamiento.
///
/// Ejemplo:
/// - Track 3 del CD1 del álbum
/// - Track 7 de la compilación japonesa

/// Representa una pista específica dentro de un lanzamiento.
///
/// A diferencia de [`crate::discography::song::Song`], que es una obra musical
/// abstracta, `ReleaseTrack` describe la *instancia concreta* de esa canción
/// dentro de un [`Release`](crate::discography::release::Release):
///
/// - orden dentro del release,
/// - archivo físico asociado,
/// - detalles técnicos,
/// - posibles cambios de título sólo para esta edición.
///
/// Un mismo `Song` puede aparecer en varios `ReleaseTrack`
/// (álbum original, compilación, remaster, edición japonesa, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseTrack {
  /// Identificador único de la pista dentro del sistema.
  pub id: ReleaseTrackId,

  // --- Relaciones ---
  /// Canción abstracta asociada.
  pub song_id: SongId,

  /// Lanzamiento al que pertenece esta pista.
  pub release_id: ReleaseId,

  // --- Metadatos dentro del release ---
  /// Número de pista (1..n) dentro de su disco.
  pub track_number: u32,

  /// Número de disco cuando el release tiene múltiples CDs o volúmenes.
  pub disc_number: u32,

  /// Título personalizado solo para este release.
  ///
  /// Algunos lanzamientos renombran pistas o añaden sufijos como:
  /// - `"Remastered"`
  /// - `"Acoustic Version"`
  /// - `"Radio Edit"`
  pub title_override: Option<String>,

  // --- Datos Técnicos ---
  /// Información técnica del audio.
  pub audio_details: AudioDetails,

  /// Información del archivo físico asociado.
  pub file_details: FileDetails,
}

/// Información técnica del audio de la pista.
///
/// Describe las características del contenido (audio) y no del archivo
/// de sistema de ficheros.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioDetails {
  /// Duración total de la pista.
  pub duration: Duration,

  /// Tasa de bits del archivo (kbps), si se puede obtener.
  pub bitrate_kbps: Option<u32>,

  /// Frecuencia de muestreo (Hz).
  pub sample_rate_hz: Option<u32>,

  /// Cantidad de canales (1 = mono, 2 = estéreo, etc.).
  pub channels: Option<u8>,

  /// Análisis técnico opcional del audio (calidad, BPM, features…).
  pub analysis: Option<AudioAnalysis>,

  /// Huella digital acústica (AcoustID, Chromaprint, etc.).
  pub fingerprint: Option<String>,
}

/// Resultado de análisis avanzado del audio.
///
/// Puede provenir de librerías DSP, servicios externos o procesos
/// internos de Gamus (por ejemplo, un analizador basado en ML).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioAnalysis {
  /// Evaluación de calidad subjetiva/estadística (SNR, artefactos, clipping…).
  pub quality: Option<AudioQuality>,

  /// Vector de características numéricas (embeddings, MFCCs, etc.).
  ///
  /// Pensado para algoritmos de recomendación, similitud o clustering.
  pub features: Option<Vec<f32>>,

  /// BPM detectado o estimado.
  pub bpm: Option<f32>,
}

/// Medida de calidad del audio.
///
/// - `score`: métrica numérica (normalizada 0.0–1.0 o escala propia).
/// - `assessment`: descripción legible para humanos.
///   Ejemplo:
///   - `"Excelente: sin pérdida perceptible"`
///   - `"Compresión fuerte: artefactos audibles"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioQuality {
  /// Puntuación cuantitativa de la calidad.
  pub score: f32,

  /// Descripción textual de la calidad percibida.
  pub assessment: String,
}

/// Describe el archivo físico en disco asociado a la pista.
///
/// Se centra en propiedades del archivo como tal, no del contenido musical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileDetails {
  /// Ruta absoluta hacia el archivo en el sistema.
  pub path: PathBuf,

  /// Tamaño del archivo en bytes.
  pub size: u64,

  /// Timestamp UNIX de última modificación (segundos desde epoch).
  ///
  /// Útil para detectar cambios y decidir si es necesario reescaneo.
  pub modified: u64,
}
