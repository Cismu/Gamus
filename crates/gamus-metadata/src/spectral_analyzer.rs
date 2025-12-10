//! # Spectral Analysis Module
//!
//! This module implements audio quality analysis by examining the frequency spectrum of a media file.
//! It primarily detects high-frequency "cutoffs" characteristic of lossy compression (e.g., MP3 128kbps cutting off at 16kHz)
//! or upsampled audio.
//!
//! **Core Logic:**
//! 1. Decodes audio stream using FFmpeg.
//! 2. Resamples to a consistent internal format (f32, Mono) required for the FFT.
//! 3. Computes the average magnitude spectrum over the duration of the file (or a sample limit).
//! 4. Analyzes the spectrum dB drop-off relative to a reference band.

use ffmpeg_next as ffmpeg;
use gamus_core::domain::release_track::{AnalysisOutcome, AudioQuality, AudioQualityReport, QualityLevel};
use num_traits::Zero;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::path::Path;
use std::sync::Arc;

/// Configuration for the spectral analyzer.
///
/// **Performance Note:** `fft_window_size` determines frequency resolution.
/// Resolution = $SampleRate / WindowSize$.
/// At 44.1kHz, 8192 bins provides ~5.38Hz resolution, sufficient for detecting specific cutoff boundaries.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
  pub fft_window_size: usize,
  /// Safety cap to prevent OOM or long processing on large files.
  pub max_analysis_duration_secs: f32,
  /// Start of the "known good" frequency band used as a baseline dB level.
  pub reference_freq_start: f32,
  pub reference_freq_end: f32,
  /// Start frequency to begin searching for amplitude drops.
  pub check_freq_start: f32,
  pub check_band_width: f32,
  pub num_check_bands: usize,
  /// The dB delta required to classify a drop as a "cutoff" rather than natural roll-off.
  pub significant_drop_db: f32,
}

impl Default for AnalysisConfig {
  fn default() -> Self {
    Self {
      fft_window_size: 8192,
      max_analysis_duration_secs: 10.0,
      reference_freq_start: 14_000.0,
      reference_freq_end: 16_000.0,
      check_freq_start: 17_000.0,
      check_band_width: 1_000.0,
      num_check_bands: 6,
      significant_drop_db: 18.0,
    }
  }
}

// --- Errors ---
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

// --- Analyzer ---

pub struct SpectralAnalyzer {
  config: AnalysisConfig,
  /// Arc used here to allow potential multi-threaded sharing of the planner if this struct is cloned,
  /// though currently used sequentially.
  fft: Arc<dyn Fft<f32>>,
  /// Pre-allocated scratch buffer for FFT computation to minimize allocations per window.
  scratch_buffer: Vec<Complex<f32>>,
  /// Buffer for input samples converted to Complex numbers.
  fft_buffer: Vec<Complex<f32>>,
  /// Pre-computed Hanning window to reduce spectral leakage.
  window: Vec<f32>,
}

impl SpectralAnalyzer {
  pub fn new(config: AnalysisConfig) -> Self {
    // SECURITY NOTE: FFmpeg initialization is global. Ensure thread-safety in the broader application context
    // if calling this from multiple threads simultaneously during startup, though `init` is generally idempotent.
    let _ = ffmpeg::init();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.fft_window_size);
    let scratch_len = fft.get_inplace_scratch_len();

    // Apodization (Windowing) is critical here. Using Hanning window
    // to minimize side lobes, trading off some main lobe width.
    let window: Vec<f32> = apodize::hanning_iter(config.fft_window_size).map(|x| x as f32).collect();

    Self {
      fft,
      scratch_buffer: vec![Complex::zero(); scratch_len],
      fft_buffer: vec![Complex::zero(); config.fft_window_size],
      window,
      config,
    }
  }

  pub fn analyze_file(&mut self, path: &Path) -> Result<AudioQuality, AnalysisError> {
    let (sample_rate, avg_spectrum) = self.compute_average_spectrum(path)?;

    //
    // The spectrum is analyzed to detect sharp drops indicating lossy compression.
    let outcome = self.detect_cutoff(&avg_spectrum, sample_rate);

    Ok(self.score_outcome(outcome))
  }

  /// Decodes audio, resamples, and computes the average magnitude spectrum.
  ///
  /// **Resource Management:**
  /// This function enforces a `max_analysis_duration_secs` to prevent DoS via massive audio files.
  /// It manually manages the FFmpeg decoder/resampler loop to ensure we feed the FFT with
  /// strictly formatted data (Packed f32 Mono).
  fn compute_average_spectrum(&mut self, path: &Path) -> Result<(u32, Vec<f32>), AnalysisError> {
    // 1. Input Context
    let mut ictx = ffmpeg::format::input(path)?;

    // 2. Stream Selection
    let input_stream = ictx.streams().best(ffmpeg::media::Type::Audio).ok_or(AnalysisError::NoCompatibleTrack)?;
    let stream_index = input_stream.index();

    // 3. Decoder Context
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
    let mut decoder = context_decoder.decoder().audio()?;
    let sample_rate = decoder.rate();

    if sample_rate == 0 {
      return Err(AnalysisError::InvalidAudioFormat);
    }

    // 4. FFT Accumulation Buffers
    // We accumulate magnitude linearly and log-transform only at the end to average correct energy levels.
    let mut magnitude_acc = vec![0.0f32; self.config.fft_window_size / 2];
    let mut window_count = 0usize;
    let mut samples_buffer = Vec::with_capacity(self.config.fft_window_size);

    // 5. Resampling Setup
    // Target: f32 Packed Mono.
    // This is a hard requirement for the `rustfft` input buffer layout we use.
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

    // Helper: Pushes samples into the window buffer and triggers FFT when full.
    let push_samples = |plane: &[f32],
                        samples_buffer: &mut Vec<f32>,
                        magnitude_acc: &mut [f32],
                        window_count: &mut usize,
                        this: &mut SpectralAnalyzer| {
      for &sample in plane {
        samples_buffer.push(sample);
        if samples_buffer.len() == this.config.fft_window_size {
          this.process_fft_window(samples_buffer, magnitude_acc);
          samples_buffer.clear();
          *window_count += 1;
        }
      }
    };

    // Helper: Lazy initialization/reconfiguration of resampler based on incoming frame properties.
    // FFmpeg streams can theoretically change parameters mid-stream.
    let ensure_resampler = |resampler: &mut Option<ffmpeg::software::resampling::Context>,
                            frame: &ffmpeg::util::frame::Audio|
     -> Result<(), ffmpeg::Error> {
      let needs_new = resampler.as_ref().map_or(true, |r| {
        r.input().rate != frame.rate()
          || r.input().format != frame.format()
          || r.input().channel_layout != frame.channel_layout()
      });
      if needs_new {
        *resampler = Some(ffmpeg::software::resampling::Context::get(
          frame.format(),
          frame.channel_layout(),
          frame.rate(),
          dst_format,
          dst_layout,
          frame.rate(),
        )?);
      }
      Ok(())
    };

    // 6. Packet Processing Loop
    for (stream, packet) in ictx.packets() {
      if stream.index() != stream_index {
        continue;
      }
      decoder.send_packet(&packet)?;
      let mut decoded_frame = ffmpeg::util::frame::Audio::empty();

      while decoder.receive_frame(&mut decoded_frame).is_ok() {
        ensure_resampler(&mut resampler, &decoded_frame)?;
        let r = resampler.as_mut().unwrap();
        let mut resampled_frame = ffmpeg::util::frame::Audio::empty();

        // We do not drain the delay here for simplicity, assuming enough overlap in continuous streams.
        let _delay = r.run(&decoded_frame, &mut resampled_frame)?;
        let plane = resampled_frame.plane::<f32>(0);

        if !plane.is_empty() {
          push_samples(plane, &mut samples_buffer, &mut magnitude_acc, &mut window_count, self);
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

    // 7. Flush Decoder
    if !stop {
      decoder.send_eof()?;
      let mut decoded_frame = ffmpeg::util::frame::Audio::empty();
      while decoder.receive_frame(&mut decoded_frame).is_ok() {
        ensure_resampler(&mut resampler, &decoded_frame)?;
        let r = resampler.as_mut().unwrap();
        let mut resampled_frame = ffmpeg::util::frame::Audio::empty();
        let _delay = r.run(&decoded_frame, &mut resampled_frame)?;
        let plane = resampled_frame.plane::<f32>(0);

        if !plane.is_empty() {
          push_samples(plane, &mut samples_buffer, &mut magnitude_acc, &mut window_count, self);
          total_samples_processed += plane.len();
        }

        if let Some(max) = max_samples {
          if total_samples_processed >= max {
            stop = true;
            break;
          }
        }
      }
    }

    // 8. Flush Resampler
    // Critical for ensuring the last few milliseconds of audio are processed.
    if !stop {
      if let Some(ref mut r) = resampler {
        let mut resampled_frame = ffmpeg::util::frame::Audio::empty();
        let mut delay = r.flush(&mut resampled_frame)?;

        loop {
          let plane = resampled_frame.plane::<f32>(0);
          if !plane.is_empty() {
            push_samples(plane, &mut samples_buffer, &mut magnitude_acc, &mut window_count, self);
          }
          if delay.is_none() {
            break;
          }
          delay = r.flush(&mut resampled_frame)?;
        }
      }
    }

    if window_count == 0 {
      return Err(AnalysisError::InvalidAudioFormat);
    }

    // 9. Normalize and Convert to dB
    // Formula: $dB = 20 \cdot \log_{10}(magnitude)$
    let avg_spectrum_db: Vec<f32> = magnitude_acc
      .iter()
      .map(|mag_sum| {
        let avg_mag = mag_sum / window_count as f32;
        20.0 * avg_mag.max(1e-10).log10()
      })
      .collect();

    Ok((sample_rate, avg_spectrum_db))
  }

  /// Processes a single window of samples:
  /// 1. Applies window function (Hanning).
  /// 2. Performs FFT.
  /// 3. Accumulates the norm (magnitude) into the accumulator.
  #[inline(always)]
  fn process_fft_window(&mut self, samples: &[f32], acc: &mut [f32]) {
    for (i, &sample) in samples.iter().enumerate() {
      self.fft_buffer[i] = Complex::new(sample * self.window[i], 0.0);
    }

    // `process_with_scratch` allows us to reuse memory, reducing GC pressure/allocations.
    self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);

    for i in 0..acc.len() {
      acc[i] += self.fft_buffer[i].norm();
    }
  }

  /// Heuristic logic to detect audio cutoffs.
  ///
  /// Compares the average dB level of a "Reference" band (e.g., 14k-16k) against
  /// sequential "Check" bands (e.g., 17k, 18k...).
  ///
  /// If `ref_db - check_db > significant_drop_db`, a cutoff is flagged.
  fn detect_cutoff(&self, spectrum_db: &[f32], sample_rate: u32) -> AnalysisOutcome {
    let nyquist = sample_rate as f32 / 2.0;
    let bin_width = nyquist / spectrum_db.len() as f32;

    // Calculates average dB in a frequency range.
    let get_db = |start: f32, end: f32| -> Option<f32> {
      let s_bin = (start / bin_width) as usize;
      let e_bin = ((end / bin_width) as usize).min(spectrum_db.len());

      if s_bin >= e_bin {
        return None;
      }

      let sum: f32 = spectrum_db[s_bin..e_bin].iter().sum();
      Some(sum / (e_bin - s_bin) as f32)
    };

    // Establish Baseline
    let ref_db = match get_db(self.config.reference_freq_start, self.config.reference_freq_end) {
      Some(v) if v > -100.0 => v, // Threshold to ignore digital silence
      _ => return AnalysisOutcome::Inconclusive("Señal muy baja o rango de referencia inválido".into()),
    };

    // Scan check bands
    for i in 0..self.config.num_check_bands {
      let start = self.config.check_freq_start + (i as f32 * self.config.check_band_width);
      let end = start + self.config.check_band_width;

      if start >= nyquist {
        break;
      }

      if let Some(band_db) = get_db(start, end) {
        if ref_db - band_db > self.config.significant_drop_db {
          return AnalysisOutcome::CutoffDetected { freq: start, ref_db, cut_db: band_db };
        }
      }
    }

    AnalysisOutcome::NoCutoffDetected {
      ref_db,
      max_freq: self.config.check_freq_start
        + (self.config.num_check_bands as f32 * self.config.check_band_width).min(nyquist),
    }
  }

  /// Maps the technical `AnalysisOutcome` to a user-facing `AudioQuality` score and report.
  fn score_outcome(&self, outcome: AnalysisOutcome) -> AudioQuality {
    let (score, assessment) = match &outcome {
      AnalysisOutcome::CutoffDetected { freq, .. } => {
        // Scoring rubric based on standard encoding cutoffs:
        // >20kHz: Near lossless/Transparency
        // ~16kHz: Standard 128kbps MP3
        // <15kHz: Low quality
        let s = match *freq {
          f if f >= 21_500.0 => 9.8,
          f if f >= 20_000.0 => 9.0,
          f if f >= 19_000.0 => 8.0,
          f if f >= 17_000.0 => 7.0,
          f if f >= 16_000.0 => 6.0,
          f if f >= 15_000.0 => 5.0,
          _ => 3.0,
        };
        (s, format!("Corte en {:.1} kHz", freq / 1000.0))
      }
      AnalysisOutcome::NoCutoffDetected { .. } => (10.0, "Espectro completo (full range)".into()),
      AnalysisOutcome::Inconclusive(reason) => (0.0, format!("Inconcluso: {}", reason)),
    };

    let report = self.build_report(&outcome, score, &assessment);

    AudioQuality { outcome, quality_score: score, assessment, report }
  }

  /// Generates the human-readable report.
  fn build_report(&self, outcome: &AnalysisOutcome, score: f32, assessment: &str) -> AudioQualityReport {
    let mut level = if score >= 9.5 {
      QualityLevel::Perfect
    } else if score >= 8.0 {
      QualityLevel::High
    } else if score >= 6.0 {
      QualityLevel::Medium
    } else {
      QualityLevel::Low
    };

    match outcome {
      AnalysisOutcome::CutoffDetected { freq, ref_db, cut_db } => {
        let drop_db = ref_db - cut_db;
        let summary = "Se detectó un recorte en las frecuencias altas del audio.".to_string();

        let details = Some(format!(
          "La banda de referencia (~{:.1}–{:.1} kHz) está alrededor de {:.1} dB. \
           A partir de {:.1} kHz el nivel cae a ≈ {:.1} dB, con una caída aproximada de {:.1} dB.",
          self.config.reference_freq_start / 1000.0,
          self.config.reference_freq_end / 1000.0,
          ref_db,
          freq / 1000.0,
          cut_db,
          drop_db
        ));

        AudioQualityReport {
          level,
          score,
          label: assessment.to_string(),
          summary,
          details,
          cutoff_freq_hz: Some(*freq),
          max_freq_hz: None,
        }
      }
      AnalysisOutcome::NoCutoffDetected { ref_db, max_freq } => {
        let summary = "No se detectó un recorte significativo en las frecuencias altas.".to_string();

        let details = Some(format!(
          "La banda de referencia (~{:.1}–{:.1} kHz) está alrededor de {:.1} dB. \
           No se observaron caídas mayores a {:.1} dB hasta aproximadamente {:.1} kHz.",
          self.config.reference_freq_start / 1000.0,
          self.config.reference_freq_end / 1000.0,
          ref_db,
          self.config.significant_drop_db,
          max_freq / 1000.0
        ));

        AudioQualityReport {
          level,
          score,
          label: assessment.to_string(),
          summary,
          details,
          cutoff_freq_hz: None,
          max_freq_hz: Some(*max_freq),
        }
      }
      AnalysisOutcome::Inconclusive(reason) => {
        level = QualityLevel::Inconclusive;
        let summary = "No fue posible determinar la calidad espectral del audio con suficiente confianza.".to_string();
        let details = Some(reason.clone());

        AudioQualityReport {
          level,
          score,
          label: assessment.to_string(),
          summary,
          details,
          cutoff_freq_hz: None,
          max_freq_hz: None,
        }
      }
    }
  }
}
