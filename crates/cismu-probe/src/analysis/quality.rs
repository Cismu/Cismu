use thiserror::Error;

use apodize::hanning_iter;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::audio::PcmStream;

// =================== Config / Constantes ===================

pub const FFT_WINDOW_SIZE: usize = 8192;
pub const REFERENCE_FREQ_START_HZ: f32 = 14_000.0;
pub const REFERENCE_FREQ_END_HZ: f32 = 16_000.0;
pub const CHECK_FREQ_START_HZ: f32 = 17_000.0;
pub const CHECK_BAND_WIDTH_HZ: f32 = 1_000.0;
pub const NUM_CHECK_BANDS: usize = 6;
pub const SIGNIFICANT_DROP_DB: f32 = 18.0;
pub const MIN_WINDOWS_TO_ANALYZE: usize = 10;

// si querés cortar por tiempo; poné 0.0 para desactivar
const MAX_ANALYSIS_DURATION_SECONDS: f32 = 10.0;

// =================== Error / Resultados ===================

#[derive(Debug, Error, Clone)]
pub enum QualityError {
    #[error("stream format unavailable (sample_rate/channels)")]
    MissingFormat,

    #[error("failed to read from PCM stream")]
    StreamRead,

    #[error("invalid channel count: {0}")]
    InvalidChannels(u16),

    #[error("analysis requires at least one sample")]
    NoData,
}

#[derive(Debug, Clone)]
pub struct QualityReport {
    pub outcome: AnalysisOutcome,
    pub score: f32,
    pub assessment: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnalysisOutcome {
    /// Caída significativa detectada → cutoff.
    CutoffDetected {
        cutoff_frequency_hz: f32,
        reference_level_db: f32,
        cutoff_band_level_db: f32,
    },
    /// No hay caída significativa en el rango analizado.
    NoCutoffDetected {
        reference_level_db: f32,
        max_analyzed_freq_hz: f32,
    },
    /// Muy pocas ventanas para confiar en el resultado.
    InconclusiveNotEnoughWindows {
        processed_windows: usize,
        required_windows: usize,
    },
    /// Error al calcular la banda de referencia (fuera de rango de Nyquist, etc.).
    InconclusiveReferenceBandError,
    /// Nivel de señal demasiado bajo en la banda de referencia.
    InconclusiveLowReferenceLevel {
        reference_level_db: f32,
    },
    InconclusiveError,
}

impl Default for AnalysisOutcome {
    fn default() -> Self {
        AnalysisOutcome::InconclusiveError
    }
}

// ============== API pública ==============

/// Analiza calidad leyendo chunks PCM del stream (f32 interleaved [-1,1]).
/// Calcula espectro promedio por ventanas, detecta cutoff y devuelve un QualityReport.
/// No usa `crate::error::Error` en la API pública.
pub fn analyze_stream(stream: &mut (dyn PcmStream + Send)) -> Result<QualityReport, QualityError> {
    let info = stream.format().ok_or(QualityError::MissingFormat)?;
    if info.channels == 0 {
        return Err(QualityError::InvalidChannels(info.channels));
    }
    let sr = info.sample_rate;
    let ch = info.channels as usize;

    // FFT setup
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_WINDOW_SIZE);
    let mut fft_buffer: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); FFT_WINDOW_SIZE];
    // scratch opcional (puede ser 0 si el plan no lo necesita)
    let mut scratch: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
    let hann: Vec<f32> = hanning_iter(FFT_WINDOW_SIZE).map(|x| x as f32).collect();

    // acumuladores
    let mut window_count: usize = 0;
    let mut spectrum_db_accum: Vec<f32> = vec![0.0; FFT_WINDOW_SIZE / 2];
    let mut mono_buf: Vec<f32> = Vec::with_capacity(FFT_WINDOW_SIZE);
    let mut fifo: Vec<f32> = Vec::with_capacity(FFT_WINDOW_SIZE);
    let mut seconds_acc = 0.0_f32;
    let mut saw_any = false;

    // Consumo por chunks
    loop {
        let chunk = stream.next_chunk().map_err(|_| QualityError::StreamRead)?;
        let Some(interleaved) = chunk else { break };
        saw_any = true;

        // mezclar a mono (promedio por frame)
        mono_buf.clear();
        for frame in interleaved.chunks_exact(ch) {
            let sum: f32 = frame.iter().copied().sum();
            mono_buf.push(sum / info.channels as f32);
        }

        // tiempo (para cortar por MAX_ANALYSIS_DURATION_SECONDS si aplica)
        seconds_acc += mono_buf.len() as f32 / sr as f32;
        if MAX_ANALYSIS_DURATION_SECONDS > 0.0 && seconds_acc >= MAX_ANALYSIS_DURATION_SECONDS {
            // recorta hasta el límite exacto para no sesgar demasiado
            let extra = ((seconds_acc - MAX_ANALYSIS_DURATION_SECONDS) * sr as f32).ceil() as usize;
            if extra < mono_buf.len() {
                mono_buf.truncate(mono_buf.len().saturating_sub(extra));
            }
        }

        // push a la FIFO y procesar ventanas completas
        fifo.extend_from_slice(&mono_buf);
        while fifo.len() >= FFT_WINDOW_SIZE {
            // ventana
            for i in 0..FFT_WINDOW_SIZE {
                let s = fifo[i] * hann[i];
                fft_buffer[i].re = s;
                fft_buffer[i].im = 0.0;
            }
            // consumir ventana
            fifo.drain(0..FFT_WINDOW_SIZE);

            // FFT
            if scratch.is_empty() {
                fft.process(&mut fft_buffer);
            } else {
                fft.process_with_scratch(&mut fft_buffer, &mut scratch);
            }

            // magnitud → dB y acumular (solo bins 0..N/2)
            for (i, bin) in fft_buffer.iter().take(FFT_WINDOW_SIZE / 2).enumerate() {
                let mag = bin.norm(); // |X[k]|
                // evitar log10(0)
                let db = 20.0 * (mag.max(1e-10)).log10();
                spectrum_db_accum[i] += db;
            }
            window_count += 1;

            // cortar si ya pasamos el límite de tiempo
            if MAX_ANALYSIS_DURATION_SECONDS > 0.0 && seconds_acc >= MAX_ANALYSIS_DURATION_SECONDS {
                fifo.clear(); // descartar resto
                break;
            }
        }

        if MAX_ANALYSIS_DURATION_SECONDS > 0.0 && seconds_acc >= MAX_ANALYSIS_DURATION_SECONDS {
            break;
        }
    }

    if !saw_any {
        return Err(QualityError::NoData);
    }

    // Promedio de espectro
    let num_bins = spectrum_db_accum.len();
    if window_count == 0 || num_bins == 0 {
        let outcome = AnalysisOutcome::InconclusiveNotEnoughWindows {
            processed_windows: window_count,
            required_windows: MIN_WINDOWS_TO_ANALYZE,
        };
        let (score, assessment) = calculate_quality_score(&outcome);
        return Ok(QualityReport {
            outcome,
            score,
            assessment,
        });
    }

    let avg_spectrum_db: Vec<f32> = spectrum_db_accum
        .into_iter()
        .map(|sum_db| sum_db / window_count as f32)
        .collect();

    // Corte de altas / score
    let outcome = calc_cutoff(window_count, &avg_spectrum_db, sr);
    let (score, assessment) = calculate_quality_score(&outcome);

    Ok(QualityReport {
        outcome,
        score,
        assessment,
    })
}

// ============== Helpers de análisis (bandas / cutoff / score) ==============

fn calculate_avg_db_in_band(start_hz: f32, end_hz: f32, freq_per_bin: f32, avg_spectrum_db: &[f32]) -> Option<f32> {
    let start_bin = (start_hz / freq_per_bin).round() as usize;
    let end_bin = (end_hz / freq_per_bin).round() as usize;
    let start_bin = start_bin.min(avg_spectrum_db.len().saturating_sub(1));
    let end_bin = end_bin.min(avg_spectrum_db.len().saturating_sub(1));

    if start_bin > end_bin || start_bin >= avg_spectrum_db.len() {
        return None;
    }
    if start_bin == end_bin {
        return Some(avg_spectrum_db[start_bin]);
    }
    let band = &avg_spectrum_db[start_bin..=end_bin];
    if band.is_empty() {
        return None;
    }
    Some(band.iter().sum::<f32>() / band.len() as f32)
}

fn calc_cutoff(window_count: usize, avg_spectrum_db: &[f32], sample_rate: u32) -> AnalysisOutcome {
    if window_count < MIN_WINDOWS_TO_ANALYZE {
        return AnalysisOutcome::InconclusiveNotEnoughWindows {
            processed_windows: window_count,
            required_windows: MIN_WINDOWS_TO_ANALYZE,
        };
    }

    let nyquist = sample_rate as f32 / 2.0;
    let num_bins = avg_spectrum_db.len();
    if num_bins == 0 {
        return AnalysisOutcome::InconclusiveReferenceBandError;
    }

    let freq_per_bin = nyquist / num_bins as f32;

    let reference_avg_db = match calculate_avg_db_in_band(
        REFERENCE_FREQ_START_HZ,
        REFERENCE_FREQ_END_HZ,
        freq_per_bin,
        avg_spectrum_db,
    ) {
        Some(db) => db,
        None => return AnalysisOutcome::InconclusiveReferenceBandError,
    };

    // Nivel mínimo aceptable en la banda de referencia (fiabilidad)
    const MIN_RELIABLE_DB_LEVEL: f32 = -100.0;
    if reference_avg_db < MIN_RELIABLE_DB_LEVEL {
        return AnalysisOutcome::InconclusiveLowReferenceLevel {
            reference_level_db: reference_avg_db,
        };
    }

    let mut max_analyzed_freq_hz = REFERENCE_FREQ_END_HZ;

    for i in 0..NUM_CHECK_BANDS {
        let band_start_hz = CHECK_FREQ_START_HZ + (i as f32 * CHECK_BAND_WIDTH_HZ);
        let band_end_hz = band_start_hz + CHECK_BAND_WIDTH_HZ;

        if band_start_hz >= nyquist {
            break;
        }

        let current_band_end_hz = band_end_hz.min(nyquist);
        max_analyzed_freq_hz = current_band_end_hz;

        if let Some(check_avg_db) =
            calculate_avg_db_in_band(band_start_hz, current_band_end_hz, freq_per_bin, avg_spectrum_db)
        {
            if reference_avg_db - check_avg_db > SIGNIFICANT_DROP_DB {
                return AnalysisOutcome::CutoffDetected {
                    cutoff_frequency_hz: band_start_hz,
                    reference_level_db: reference_avg_db,
                    cutoff_band_level_db: check_avg_db,
                };
            }
        }
    }

    AnalysisOutcome::NoCutoffDetected {
        reference_level_db: reference_avg_db,
        max_analyzed_freq_hz,
    }
}

fn calculate_quality_score(outcome: &AnalysisOutcome) -> (f32, String) {
    match outcome {
        AnalysisOutcome::CutoffDetected {
            cutoff_frequency_hz, ..
        } => {
            let score = if *cutoff_frequency_hz >= 21_500.0 {
                9.8
            } else if *cutoff_frequency_hz >= 20_500.0 {
                9.0
            } else if *cutoff_frequency_hz >= 19_500.0 {
                8.0
            } else if *cutoff_frequency_hz >= 18_500.0 {
                7.0
            } else if *cutoff_frequency_hz >= 17_500.0 {
                6.0
            } else if *cutoff_frequency_hz >= 16_500.0 {
                5.0
            } else if *cutoff_frequency_hz >= 15_500.0 {
                4.0
            } else {
                3.0
            };

            let assessment = match score {
                s if s >= 9.5 => "Excellent",
                s if s >= 8.5 => "Very High",
                s if s >= 7.5 => "High",
                s if s >= 6.5 => "Good",
                s if s >= 5.5 => "Medium-High",
                s if s >= 4.5 => "Medium",
                s if s >= 3.5 => "Medium-Low",
                _ => "Low",
            }
            .to_string();

            (score, assessment)
        }
        AnalysisOutcome::NoCutoffDetected { .. } => (10.0, "Perfect".to_string()),
        AnalysisOutcome::InconclusiveNotEnoughWindows {
            processed_windows,
            required_windows,
        } => (
            0.0,
            format!(
                "Incomplete analysis (insufficient windows {processed_windows}/{required_windows}). Quality not determined."
            ),
        ),
        AnalysisOutcome::InconclusiveReferenceBandError => (
            0.0,
            "Incomplete analysis (error in reference band). Quality not determined.".to_string(),
        ),
        AnalysisOutcome::InconclusiveLowReferenceLevel { reference_level_db } => (
            0.0,
            format!("Analysis inconclusive (low reference level {reference_level_db:.1} dB). Quality not determined."),
        ),
        _ => (0.0, "Analysis inconclusive".to_string()),
    }
}
