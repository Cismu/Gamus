/// Ajustes de cómo se calcula el ruido de fondo.
#[derive(Debug, Clone)]
pub struct NoiseConfig {
  /// Umbral base de ruido (dB). Todo lo que quede por debajo se considera "no música".
  pub base_floor_db: f32,
  /// Margen dinámico: noise_floor = max(base_floor_db, global_max_db - dynamic_margin_db)
  pub dynamic_margin_db: f32,
  /// Umbral de desviación estándar para considerar que el espectro es plano (dB).
  pub flat_spectrum_std_threshold_db: f32,
}

impl Default for NoiseConfig {
  fn default() -> Self {
    Self {
      base_floor_db: -65.0,
      dynamic_margin_db: 70.0, // antes estaba “hardcoded” en detect_cutoff
      flat_spectrum_std_threshold_db: 4.0,
    }
  }
}

/// Ajustes del reverse scan en alta frecuencia.
#[derive(Debug, Clone)]
pub struct ReverseScanConfig {
  /// Ancho de banda usado en el reverse scan (Hz).
  pub band_width_hz: f32,
  /// Margen desde Nyquist para considerar que hay recorte (Hz).
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
#[derive(Debug, Clone)]
pub struct ScoringConfig {
  /// Pares (freq_hz, score) ordenados de mayor freq a menor.
  /// Se toma el primero cuyo umbral se cumpla.
  pub cutoff_bands: Vec<(f32, f32)>,
  /// Puntación si ningún umbral de cutoff encaja.
  pub cutoff_fallback_score: f32,
  /// Puntuaciones cuando no hay cutoff: (>=21k, >=20k, resto).
  pub full_band_scores: (f32, f32, f32),
}

impl ScoringConfig {
  pub fn score_for_cutoff(&self, freq_hz: f32) -> f32 {
    for (threshold_hz, score) in &self.cutoff_bands {
      if freq_hz >= *threshold_hz {
        return *score;
      }
    }
    self.cutoff_fallback_score
  }

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
#[derive(Debug, Clone)]
pub struct BitrateSafetyConfig {
  // Umbrales en bps
  pub very_low_bps_max: i64, // < 80 kbps
  pub low_bps_max: i64,      // 80–128
  pub medium_bps_max: i64,   // 128–192
  pub high_bps_max: i64,     // 192–256
  pub lossy_bps_max: i64,    // 256–400

  // Caps de score
  pub very_low_score_cap: f32,
  pub low_score_cap: f32,
  pub medium_score_cap: f32,
  pub high_score_cap: f32,
  pub lossy_score_cap: f32,
}

impl BitrateSafetyConfig {
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
/// Todo lo que antes eran números mágicos ahora vive aquí.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
  pub fft_window_size: usize,
  pub max_analysis_duration_secs: f32,
  pub noise: NoiseConfig,
  pub reverse_scan: ReverseScanConfig,
  pub scoring: ScoringConfig,
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

/// Builder para no tener que tocar todos los campos a mano.
#[derive(Debug, Clone)]
pub struct AnalysisConfigBuilder {
  inner: AnalysisConfig,
}

impl AnalysisConfigBuilder {
  pub fn new() -> Self {
    Self { inner: AnalysisConfig::default() }
  }

  pub fn fft_window_size(mut self, size: usize) -> Self {
    self.inner.fft_window_size = size;
    self
  }

  pub fn max_analysis_duration_secs(mut self, secs: f32) -> Self {
    self.inner.max_analysis_duration_secs = secs;
    self
  }

  pub fn noise_floor_db(mut self, db: f32) -> Self {
    self.inner.noise.base_floor_db = db;
    self
  }

  pub fn dynamic_noise_margin_db(mut self, db: f32) -> Self {
    self.inner.noise.dynamic_margin_db = db;
    self
  }

  pub fn reverse_scan_band_width_hz(mut self, hz: f32) -> Self {
    self.inner.reverse_scan.band_width_hz = hz;
    self
  }

  pub fn margin_from_nyquist_hz(mut self, hz: f32) -> Self {
    self.inner.reverse_scan.margin_from_nyquist_hz = hz;
    self
  }

  /// Si quieres exponer tuning más fino del scoring:
  pub fn scoring(mut self, scoring: ScoringConfig) -> Self {
    self.inner.scoring = scoring;
    self
  }

  pub fn bitrate_safety(mut self, bs: BitrateSafetyConfig) -> Self {
    self.inner.bitrate_safety = bs;
    self
  }

  pub fn build(self) -> AnalysisConfig {
    self.inner
  }
}

impl AnalysisConfig {
  pub fn builder() -> AnalysisConfigBuilder {
    AnalysisConfigBuilder::new()
  }
}
