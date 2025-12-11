//! Configuración del analizador espectral.
//!
//! La idea es sacar todos los “magic numbers” del código y hacerlos
//! explicitamente tuneables desde configuración o tests.

/// Ajustes de cómo se calcula el ruido de fondo.
///
/// Se usa para distinguir entre energía “real” en alta frecuencia y
/// ruido / silencio numérico del espectro.
#[derive(Debug, Clone)]
pub struct NoiseConfig {
  /// Umbral base de ruido (dB).
  ///
  /// Todo lo que esté por debajo se considera ruido de fondo, incluso
  /// aunque el contenido suba o baje. Sube este valor si el analizador
  /// es demasiado “optimista” detectando energía residual en altas.
  pub base_floor_db: f32,

  /// Margen dinámico respecto al máximo global:
  /// `noise_floor = max(base_floor_db, global_max_db - dynamic_margin_db)`.
  ///
  /// Permite adaptar el floor al nivel real de la pista. Valores altos
  /// hacen el floor más bajo (más agresivo encontrando energía débil),
  /// valores bajos lo acercan al máximo (más conservador).
  pub dynamic_margin_db: f32,
}

impl Default for NoiseConfig {
  fn default() -> Self {
    Self {
      base_floor_db: -65.0,
      dynamic_margin_db: 70.0, // antes estaba “hardcoded” en detect_cutoff
    }
  }
}

/// Ajustes del reverse scan en alta frecuencia.
///
/// Controla cómo buscamos la presencia/ausencia de energía cerca de Nyquist.
#[derive(Debug, Clone)]
pub struct ReverseScanConfig {
  /// Ancho de banda usado en el reverse scan (Hz).
  ///
  /// El espectro se recorre en ventanas [f - band_width_hz, f], empezando
  /// desde Nyquist. Ventanas más anchas suavizan el ruido local pero
  /// pierden resolución en freq de corte.
  pub band_width_hz: f32,

  /// Margen desde Nyquist para considerar que hay recorte (Hz).
  ///
  /// Si la última banda con energía significativa está más lejos que este
  /// margen, se interpreta como cutoff (p.ej. compresión con pérdida).
  /// Sube este valor para ser más estricto, bájalo para ser más permisivo.
  pub margin_from_nyquist_hz: f32,
}

impl Default for ReverseScanConfig {
  fn default() -> Self {
    Self {
      band_width_hz: 1_000.0,
      margin_from_nyquist_hz: 1_500.0, // antes: const “perdida” por ahí
    }
  }
}

/// Cómo se mapea el cutoff (o ausencia de cutoff) a una puntuación numérica.
///
/// Separa la detección acústica de la política de puntuación.
#[derive(Debug, Clone)]
pub struct ScoringConfig {
  /// Pares `(freq_hz, score)` ordenados de mayor freq a menor.
  ///
  /// Se toma el primer score cuyo `freq_hz` sea <= freq de cutoff detectado.
  /// Esto permite redefinir fácilmente la tabla de conversión sin tocar lógica.
  pub cutoff_bands: Vec<(f32, f32)>,

  /// Puntuación si ningún umbral de `cutoff_bands` encaja.
  ///
  /// Útil como valor por defecto para casos “raros” de cutoff.
  pub cutoff_fallback_score: f32,

  /// Puntuaciones cuando NO hay cutoff: `(>=21k, >=20k, resto)`.
  ///
  /// Se usa cuando el análisis considera el espectro “full band”.
  pub full_band_scores: (f32, f32, f32),
}

impl ScoringConfig {
  /// Devuelve la puntuación asociada a una frecuencia de cutoff detectada.
  ///
  /// Asume que `cutoff_bands` está ordenado de freq mayor a menor.
  pub fn score_for_cutoff(&self, freq_hz: f32) -> f32 {
    for (threshold_hz, score) in &self.cutoff_bands {
      if freq_hz >= *threshold_hz {
        return *score;
      }
    }
    self.cutoff_fallback_score
  }

  /// Devuelve la puntuación asociada a un espectro completo (sin cutoff).
  ///
  /// Usa los tres tramos configurados: `>=21k`, `>=20k` y resto.
  pub fn score_for_full_band(&self, max_freq_hz: f32) -> f32 {
    let (s_21k, s_20k, s_default) = self.full_band_scores;
    if max_freq_hz >= 21_000.0 {
      s_21k
    } else if max_freq_hz >= 20_000.0 {
      s_20k
    } else {
      s_default
    }
  }
}

impl Default for ScoringConfig {
  fn default() -> Self {
    Self {
      // Replica tu tabla original:
      cutoff_bands: vec![
        (21_000.0, 10.0), // lossless 44.1/48/96
        (20_000.0, 9.5),
        (18_000.0, 8.0), // MP3 320/V0
        (16_500.0, 7.0), // ~192 kbps
        (15_000.0, 6.0), // ~128 kbps
        (13_000.0, 4.5), // malo / streaming chusco
        (11_500.0, 2.0), // 64 kbps, etc.
      ],
      cutoff_fallback_score: 4.0,
      full_band_scores: (10.0, 9.5, 9.0),
    }
  }
}

/// Reglas de seguridad basadas en bitrate (cap de la nota).
///
/// Evita que una pista de bitrate muy bajo obtenga una nota
/// “imposible” solo por cómo cae el espectro.
#[derive(Debug, Clone)]
pub struct BitrateSafetyConfig {
  // Umbrales en bps: definen los tramos de bitrate.
  pub very_low_bps_max: i64, // < 80 kbps
  pub low_bps_max: i64,      // 80–128
  pub medium_bps_max: i64,   // 128–192
  pub high_bps_max: i64,     // 192–256
  pub lossy_bps_max: i64,    // 256–400

  // Caps de score por tramo.
  pub very_low_score_cap: f32,
  pub low_score_cap: f32,
  pub medium_score_cap: f32,
  pub high_score_cap: f32,
  pub lossy_score_cap: f32,
}

impl BitrateSafetyConfig {
  /// Aplica un límite superior a `score` en función del bitrate.
  ///
  /// Solo reduce la puntuación, nunca la aumenta. Añade una nota al
  /// `assessment` cuando el bitrate es sospechosamente bajo.
  pub fn apply_cap(&self, bitrate_bps: i64, score: &mut f32, assessment: &mut String) {
    if bitrate_bps <= 0 {
      return;
    }

    if bitrate_bps < self.very_low_bps_max {
      if *score > self.very_low_score_cap {
        *score = self.very_low_score_cap;
        assessment.push_str(" (Bitrate muy bajo)");
      }
    } else if bitrate_bps < self.low_bps_max {
      if *score > self.low_score_cap {
        *score = self.low_score_cap;
        assessment.push_str(" (Bitrate bajo)");
      }
    } else if bitrate_bps < self.medium_bps_max {
      if *score > self.medium_score_cap {
        *score = self.medium_score_cap;
      }
    } else if bitrate_bps < self.high_bps_max {
      if *score > self.high_score_cap {
        *score = self.high_score_cap;
      }
    } else if bitrate_bps < self.lossy_bps_max {
      if *score > self.lossy_score_cap {
        *score = self.lossy_score_cap;
      }
    } else {
      // ≥ lossy_bps_max → dejamos pasar (probable lossless)
    }
  }
}

impl Default for BitrateSafetyConfig {
  fn default() -> Self {
    Self {
      very_low_bps_max: 80_000,
      low_bps_max: 128_000,
      medium_bps_max: 192_000,
      high_bps_max: 256_000,
      lossy_bps_max: 400_000,
      very_low_score_cap: 3.0,
      low_score_cap: 5.5,
      medium_score_cap: 7.5,
      high_score_cap: 8.5,
      lossy_score_cap: 9.0,
    }
  }
}

/// Configuración de análisis de espectro completa.
///
/// Punto único de entrada para ajustar el comportamiento del
/// analizador sin tocar la lógica de `SpectralAnalyzer`.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
  /// Tamaño de ventana FFT (en muestras).
  ///
  /// Afecta resolución en frecuencia y coste computacional. Debe ser
  /// consistente con el plan FFT y el tamaño de los buffers internos.
  pub fft_window_size: usize,

  /// Máxima duración de audio a analizar (en segundos).
  ///
  /// Permite acotar el tiempo de análisis en pistas muy largas para
  /// evitar tiempos de CPU desproporcionados. `<= 0` desactiva el límite.
  pub max_analysis_duration_secs: f32,

  /// Parámetros de cálculo del ruido de fondo.
  pub noise: NoiseConfig,

  /// Parámetros del reverse scan de alta frecuencia.
  pub reverse_scan: ReverseScanConfig,

  /// Política de mapeo a puntuación.
  pub scoring: ScoringConfig,

  /// Safety net basado en bitrate.
  pub bitrate_safety: BitrateSafetyConfig,
}

impl Default for AnalysisConfig {
  fn default() -> Self {
    Self {
      fft_window_size: 8192,
      max_analysis_duration_secs: 15.0,
      noise: NoiseConfig::default(),
      reverse_scan: ReverseScanConfig::default(),
      scoring: ScoringConfig::default(),
      bitrate_safety: BitrateSafetyConfig::default(),
    }
  }
}

/// Builder para `AnalysisConfig` para evitar tocar todos los campos a mano.
///
/// Pensado para tests, estrategias de A/B y tuning avanzado.
#[derive(Debug, Clone)]
pub struct AnalysisConfigBuilder {
  inner: AnalysisConfig,
}

impl AnalysisConfigBuilder {
  /// Crea un builder con `AnalysisConfig::default()`.
  pub fn new() -> Self {
    Self { inner: AnalysisConfig::default() }
  }

  /// Ajusta el tamaño de ventana FFT.
  pub fn fft_window_size(mut self, size: usize) -> Self {
    self.inner.fft_window_size = size;
    self
  }

  /// Ajusta la duración máxima de análisis (segundos).
  pub fn max_analysis_duration_secs(mut self, secs: f32) -> Self {
    self.inner.max_analysis_duration_secs = secs;
    self
  }

  /// Ajusta el floor de ruido base (dB).
  pub fn noise_floor_db(mut self, db: f32) -> Self {
    self.inner.noise.base_floor_db = db;
    self
  }

  /// Ajusta el margen dinámico de ruido (dB).
  pub fn dynamic_noise_margin_db(mut self, db: f32) -> Self {
    self.inner.noise.dynamic_margin_db = db;
    self
  }

  /// Ajusta el ancho de banda del reverse scan (Hz).
  pub fn reverse_scan_band_width_hz(mut self, hz: f32) -> Self {
    self.inner.reverse_scan.band_width_hz = hz;
    self
  }

  /// Ajusta el margen desde Nyquist para cutoff (Hz).
  pub fn margin_from_nyquist_hz(mut self, hz: f32) -> Self {
    self.inner.reverse_scan.margin_from_nyquist_hz = hz;
    self
  }

  /// Permite inyectar una política de scoring completa.
  pub fn scoring(mut self, scoring: ScoringConfig) -> Self {
    self.inner.scoring = scoring;
    self
  }

  /// Permite inyectar una política de caps por bitrate distinta.
  pub fn bitrate_safety(mut self, bs: BitrateSafetyConfig) -> Self {
    self.inner.bitrate_safety = bs;
    self
  }

  /// Consume el builder y devuelve la configuración final.
  pub fn build(self) -> AnalysisConfig {
    self.inner
  }
}

impl AnalysisConfig {
  /// Crea un `AnalysisConfigBuilder` partiendo de los valores por defecto.
  pub fn builder() -> AnalysisConfigBuilder {
    AnalysisConfigBuilder::new()
  }
}
