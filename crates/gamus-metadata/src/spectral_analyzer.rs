//! Implementación del analizador espectral basado en FFmpeg + rustfft.
//!
//! Responsabilidades principales:
//! - Leer audio de fichero usando FFmpeg.
//! - Convertir a mono float32 y limitar duración de análisis.
//! - Acumular espectros de ventanas FFT con ventana de Hann.
//! - Detectar cutoff en altas frecuencias.
//! - Mapear resultado a `AudioQuality` + `AudioQualityReport`.

use ffmpeg_next as ffmpeg;

use gamus_core::domain::release_track::{AnalysisOutcome, AudioQuality, AudioQualityReport, QualityLevel};
use num_traits::Zero;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::path::Path;
use std::sync::Arc;

use crate::config::AnalysisConfig;

/// Errores posibles durante el análisis espectral.
///
/// Envuelven errores de E/S y de FFmpeg, y validan que el stream de
/// audio sea decodificable y no esté vacío.
#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
  #[error("File open error: {0}")]
  FileOpen(#[from] std::io::Error),

  #[error("FFmpeg error: {0}")]
  FFmpeg(#[from] ffmpeg::Error),

  #[error("No compatible audio track found")]
  NoCompatibleTrack,

  #[error("Invalid audio format or empty stream")]
  InvalidAudioFormat,
}

/// Analizador espectral de una sola pasada sobre el archivo.
///
/// El estado interno (`fft_buffer`, `scratch_buffer`, `window`) se
/// reutiliza entre análisis para minimizar asignaciones.
pub struct SpectralAnalyzer {
  config: AnalysisConfig,
  fft: Arc<dyn Fft<f32>>,
  scratch_buffer: Vec<Complex<f32>>,
  fft_buffer: Vec<Complex<f32>>,
  window: Vec<f32>,
}

impl SpectralAnalyzer {
  /// Crea un analizador con `AnalysisConfig::default()`.
  pub fn new() -> Self {
    Self::new_with_config(AnalysisConfig::default())
  }

  /// Crea un analizador con una configuración explícita.
  ///
  /// Útil para tests, tuning o entornos con requisitos especiales de
  /// rendimiento/precisión.
  pub fn new_with_config(config: AnalysisConfig) -> Self {
    let _ = ffmpeg::init();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.fft_window_size);
    let scratch_len = fft.get_inplace_scratch_len();

    let window: Vec<f32> = apodize::hanning_iter(config.fft_window_size).map(|x| x as f32).collect();

    Self {
      fft,
      scratch_buffer: vec![Complex::zero(); scratch_len],
      fft_buffer: vec![Complex::zero(); config.fft_window_size],
      window,
      config,
    }
  }

  /// API pública principal: analiza un fichero y devuelve `AudioQuality`.
  ///
  /// El flujo es:
  /// 1. Cálculo de espectro promedio (por ventanas FFT).
  /// 2. Detección de cutoff / full band.
  /// 3. Scoring + caps por bitrate + reporte de alto nivel.
  pub fn analyze_file(&mut self, path: &Path) -> Result<AudioQuality, AnalysisError> {
    let (sample_rate, avg_spectrum, bitrate_opt) = self.compute_average_spectrum(path)?;
    let outcome = self.detect_cutoff(&avg_spectrum, sample_rate);
    Ok(self.score_outcome(outcome, bitrate_opt))
  }

  /// Calcula el espectro medio (en dB) del fichero.
  ///
  /// - Escoge el mejor stream de audio con FFmpeg.
  /// - Re-muestrea a mono float32.
  /// - Aplica ventanas FFT con Hann.
  /// - Promedia el módulo del espectro en todas las ventanas.
  ///
  /// Respeta `max_analysis_duration_secs` para acotar el trabajo.
  fn compute_average_spectrum(&mut self, path: &Path) -> Result<(u32, Vec<f32>, Option<i64>), AnalysisError> {
    let mut ictx = ffmpeg::format::input(path)?;
    let input_stream = ictx.streams().best(ffmpeg::media::Type::Audio).ok_or(AnalysisError::NoCompatibleTrack)?;
    let stream_index = input_stream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
    let mut decoder = context_decoder.decoder().audio()?;
    let sample_rate = decoder.rate();

    if sample_rate == 0 {
      return Err(AnalysisError::InvalidAudioFormat);
    }

    let decoder_bitrate = decoder.bit_rate();
    let bitrate_opt = if decoder_bitrate > 0 { Some(decoder_bitrate as i64) } else { None };

    let mut magnitude_acc = vec![0.0f32; self.config.fft_window_size / 2];
    let mut window_count = 0usize;
    let mut samples_buffer = Vec::with_capacity(self.config.fft_window_size);

    let dst_format = ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed);
    let dst_layout = ffmpeg::util::channel_layout::ChannelLayout::MONO;
    let mut resampler: Option<ffmpeg::software::resampling::Context> = None;

    let max_samples = if self.config.max_analysis_duration_secs > 0.0 {
      Some((self.config.max_analysis_duration_secs * sample_rate as f32) as usize)
    } else {
      None
    };

    let mut total_samples_processed = 0usize;
    let mut stop = false;

    // Función local para procesar una tira de samples mono.
    let mut process_plane = |plane: &[f32], analyzer: &mut SpectralAnalyzer| {
      for &sample in plane {
        samples_buffer.push(sample);
        if samples_buffer.len() == analyzer.config.fft_window_size {
          analyzer.process_fft_window(&samples_buffer, &mut magnitude_acc);
          samples_buffer.clear();
          window_count += 1;
        }
      }
    };

    for (stream, packet) in ictx.packets() {
      if stream.index() != stream_index {
        continue;
      }

      decoder.send_packet(&packet)?;
      let mut decoded = ffmpeg::util::frame::Audio::empty();

      while decoder.receive_frame(&mut decoded).is_ok() {
        if resampler.is_none() || resampler.as_ref().unwrap().input().rate != decoded.rate() {
          resampler = Some(ffmpeg::software::resampling::Context::get(
            decoded.format(),
            decoded.channel_layout(),
            decoded.rate(),
            dst_format,
            dst_layout,
            decoded.rate(),
          )?);
        }

        let r = resampler.as_mut().unwrap();
        let mut resampled = ffmpeg::util::frame::Audio::empty();
        let _ = r.run(&decoded, &mut resampled)?;

        let plane = resampled.plane::<f32>(0);
        if !plane.is_empty() {
          process_plane(plane, self);
          total_samples_processed += plane.len();
        }

        if let Some(max) = max_samples {
          if total_samples_processed >= max {
            stop = true;
            break;
          }
        }
      }

      if stop {
        break;
      }
    }

    // Flush final para vaciar buffers de decoder / resampler.
    if !stop {
      decoder.send_eof()?;
      let mut decoded = ffmpeg::util::frame::Audio::empty();

      while decoder.receive_frame(&mut decoded).is_ok() {
        let r = resampler.as_mut().unwrap();
        let mut resampled = ffmpeg::util::frame::Audio::empty();
        let _ = r.run(&decoded, &mut resampled)?;
        process_plane(resampled.plane::<f32>(0), self);
      }

      if let Some(ref mut r) = resampler {
        let mut resampled = ffmpeg::util::frame::Audio::empty();
        while r.flush(&mut resampled).is_ok() {
          let plane = resampled.plane::<f32>(0);
          if plane.is_empty() {
            break;
          }
          process_plane(plane, self);
        }
      }
    }

    if window_count == 0 {
      return Err(AnalysisError::InvalidAudioFormat);
    }

    let avg_spectrum_db: Vec<f32> = magnitude_acc
      .iter()
      .map(|mag_sum| {
        let avg_mag = mag_sum / window_count as f32;
        20.0 * avg_mag.max(1e-10).log10()
      })
      .collect();

    Ok((sample_rate, avg_spectrum_db, bitrate_opt))
  }

  /// Media en dB del espectro en una banda [start, end] (Hz).
  ///
  /// Devuelve `None` si la banda queda fuera de Nyquist o no hay bins suficientes.
  fn band_db(&self, spectrum_db: &[f32], sample_rate: u32, start: f32, end: f32) -> Option<f32> {
    let nyquist = sample_rate as f32 / 2.0;
    if start >= nyquist {
      return None;
    }
    let end = end.min(nyquist);

    let bin_width = nyquist / spectrum_db.len() as f32;
    let s_bin = (start / bin_width) as usize;
    let e_bin = ((end / bin_width) as usize).min(spectrum_db.len());

    if s_bin >= e_bin {
      return None;
    }

    let sum: f32 = spectrum_db[s_bin..e_bin].iter().sum();
    Some(sum / (e_bin - s_bin) as f32)
  }

  /// Procesa una ventana FFT y acumula el módulo del espectro en `acc`.
  ///
  /// Aplica ventana de Hann precomputada y usa FFT in-place con scratch buffer.
  fn process_fft_window(&mut self, samples: &[f32], acc: &mut [f32]) {
    for (i, &sample) in samples.iter().enumerate() {
      self.fft_buffer[i] = Complex::new(sample * self.window[i], 0.0);
    }
    self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);
    for i in 0..acc.len() {
      acc[i] += self.fft_buffer[i].norm();
    }
  }

  // ---- Lógica de cutoff con la nueva config ----

  /// Detecta cutoff o espectro completo a partir del espectro medio.
  ///
  /// Estrategia:
  /// - Calcula un noise floor (base + margen dinámico).
  /// - Escanea en reversa desde Nyquist en bandas configurables.
  /// - La última banda con energía por encima del floor define `found_cutoff_freq`.
  /// - Si está suficientemente lejos de Nyquist (`margin_from_nyquist_hz`), se considera cutoff.
  fn detect_cutoff(&self, spectrum_db: &[f32], sample_rate: u32) -> AnalysisOutcome {
    let nyquist = sample_rate as f32 / 2.0;

    let global_max = spectrum_db.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let mut noise_floor = self.config.noise.base_floor_db;

    if global_max.is_finite() {
      let dyn_floor = global_max - self.config.noise.dynamic_margin_db;
      noise_floor = noise_floor.max(dyn_floor);
    }

    let step_hz = self.config.reverse_scan.band_width_hz.max(100.0);

    let mut found_cutoff_freq = 0.0;
    let mut max_db_found = -100.0;

    let mut f = (nyquist / step_hz).floor() * step_hz;
    while f >= step_hz {
      let start = f - step_hz;
      let end = f;

      if let Some(db) = self.band_db(spectrum_db, sample_rate, start, end) {
        if db > noise_floor {
          found_cutoff_freq = end;
          max_db_found = db;
          break;
        }
      }

      f -= step_hz;
    }

    if found_cutoff_freq <= 0.0 {
      return AnalysisOutcome::Inconclusive("Audio silente o sin energía significativa en alta frecuencia".into());
    }

    // Margen parametrizado
    if nyquist - found_cutoff_freq > self.config.reverse_scan.margin_from_nyquist_hz {
      AnalysisOutcome::CutoffDetected { freq: found_cutoff_freq, ref_db: max_db_found, cut_db: noise_floor }
    } else {
      AnalysisOutcome::NoCutoffDetected { ref_db: max_db_found, max_freq: found_cutoff_freq }
    }
  }

  /// Asigna una puntuación al resultado del análisis y aplica caps por bitrate.
  fn score_outcome(&self, outcome: AnalysisOutcome, bitrate: Option<i64>) -> AudioQuality {
    let (mut score, mut assessment) = match &outcome {
      AnalysisOutcome::CutoffDetected { freq, .. } => {
        let s = self.config.scoring.score_for_cutoff(*freq);
        (s, format!("Corte espectral en {:.1} kHz", freq / 1000.0))
      }
      AnalysisOutcome::NoCutoffDetected { max_freq, .. } => {
        let s = self.config.scoring.score_for_full_band(*max_freq);
        (s, "Espectro completo".into())
      }
      AnalysisOutcome::Inconclusive(reason) => (0.0, format!("Error: {}", reason)),
    };

    // SAFETY NET de bitrate, ahora encapsulado en BitrateSafetyConfig
    if let Some(br) = bitrate {
      self.config.bitrate_safety.apply_cap(br, &mut score, &mut assessment);
    }

    let report = self.build_report(&outcome, score, &assessment);
    AudioQuality { outcome, quality_score: score, assessment, report }
  }

  /// Construye el `AudioQualityReport` de alto nivel a partir del resultado.
  fn build_report(&self, outcome: &AnalysisOutcome, score: f32, assessment: &str) -> AudioQualityReport {
    let level = if score >= 9.5 {
      QualityLevel::Perfect
    } else if score >= 8.0 {
      QualityLevel::High
    } else if score >= 5.5 {
      QualityLevel::Medium
    } else {
      QualityLevel::Low
    };

    match outcome {
      AnalysisOutcome::CutoffDetected { freq, ref_db, .. } => AudioQualityReport {
        level,
        score,
        label: assessment.to_string(),
        summary: "Se detectó pérdida de frecuencias agudas.".into(),
        details: Some(format!(
          "La señal de audio cae abruptamente a partir de los {:.1} kHz (Nivel aprox: {:.1} dB). \
                     Esto es indicativo de compresión con pérdida (MP3/AAC).",
          freq / 1000.0,
          ref_db
        )),
        cutoff_freq_hz: Some(*freq),
        max_freq_hz: None,
      },
      AnalysisOutcome::NoCutoffDetected { max_freq, ref_db } => AudioQualityReport {
        level,
        score,
        label: assessment.to_string(),
        summary: "Excelente respuesta en frecuencia.".into(),
        details: Some(format!(
          "La señal se extiende hasta los {:.1} kHz sin caídas significativas (Nivel final: {:.1} dB). \
                     Consistente con audio Lossless o alta calidad.",
          max_freq / 1000.0,
          ref_db
        )),
        cutoff_freq_hz: None,
        max_freq_hz: Some(*max_freq),
      },
      AnalysisOutcome::Inconclusive(r) => AudioQualityReport {
        level: QualityLevel::Inconclusive,
        score: 0.0,
        label: "Error".into(),
        summary: "No se pudo analizar".into(),
        details: Some(r.clone()),
        cutoff_freq_hz: None,
        max_freq_hz: None,
      },
    }
  }
}
