use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use rustfft::{FftPlanner, num_complex::Complex};

use apodize::hanning_iter;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const FFT_WINDOW_SIZE: usize = 8192;
pub const REFERENCE_FREQ_START_HZ: f32 = 14_000.0;
pub const REFERENCE_FREQ_END_HZ: f32 = 16_000.0;
pub const CHECK_FREQ_START_HZ: f32 = 17_000.0;
pub const CHECK_BAND_WIDTH_HZ: f32 = 1_000.0;
pub const NUM_CHECK_BANDS: usize = 6;
pub const SIGNIFICANT_DROP_DB: f32 = 18.0;
pub const MIN_WINDOWS_TO_ANALYZE: usize = 10;
const MAX_ANALYSIS_DURATION_SECONDS: f32 = 10.0;

#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error(
        "The audio file does not have enough channels to calculate the sound quality. it is probably not playable."
    )]
    InvalidChannelNumber,

    #[error("The track has no sample rate")]
    InvalidSampleRate,

    #[error("Failed to open file: {path}")]
    FileOpen {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to probe file format")]
    ProbeFormat(#[source] SymphoniaError),

    #[error("No compatible audio track found in the file")]
    NoCompatibleTrack,

    #[error("Failed to create decoder for codec: {codec:?}")]
    CreateDecoder {
        codec: symphonia::core::codecs::CodecType,
        #[source]
        source: SymphoniaError,
    },

    #[error("Error when generating the Hann window: wrong size {0} vs {1}")]
    HannWindowError(usize, usize),

    #[error("Failed to read audio packet")]
    PacketReadError(#[source] SymphoniaError),

    #[error("Unrecoverable decoder error")]
    DecoderError(#[source] SymphoniaError),

    #[error("Buffer size overflow: {0} * {1} exceeds usize max")]
    BufferSizeOverflow(usize, usize),
}

#[derive(Debug, Clone, Default)]
pub struct AudioAnalysis {
    #[allow(dead_code)]
    pub spectral_analysis: AnalysisOutcome,
    pub quality_score: f32,
    pub overall_assessment: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnalysisOutcome {
    /// A significant drop was detected, indicating a cutoff.
    CutoffDetected {
        /// The starting frequency (Hz) of the band where the drop was first detected.
        cutoff_frequency_hz: f32,
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
        /// The average dB level in the band where the cutoff was detected.
        cutoff_band_level_db: f32,
    },
    /// No significant drop was detected within the analyzed frequency range.
    NoCutoffDetected {
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
        /// The highest frequency (Hz) analyzed.
        max_analyzed_freq_hz: f32,
    },
    /// Analysis could not be performed reliably due to insufficient audio data.
    InconclusiveNotEnoughWindows {
        /// Number of windows processed.
        processed_windows: usize,
        /// Minimum number of windows required for analysis.
        required_windows: usize,
    },
    /// Analysis failed because the reference dB level could not be calculated.
    /// This might happen if the reference frequency range is outside the spectrum data.
    InconclusiveReferenceBandError,
    /// Analysis is considered unreliable because the signal level in the reference band is too low.
    InconclusiveLowReferenceLevel {
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
    },

    InconclusiveError,
}

impl Default for AnalysisOutcome {
    fn default() -> Self {
        AnalysisOutcome::InconclusiveError
    }
}

fn setup_symphonia(path: &PathBuf) -> Result<(Box<dyn FormatReader>, Box<dyn Decoder>)> {
    let file = File::open(path).map_err(|e| AnalysisError::FileOpen {
        path: path.clone(),
        source: e,
    })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = Hint::new();
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(AnalysisError::ProbeFormat)?;

    let format_reader = probed.format;

    let track = format_reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(AnalysisError::NoCompatibleTrack)?;

    let codec_params = &track.codec_params;
    let dec_opts: DecoderOptions = Default::default();

    let decoder = symphonia::default::get_codecs()
        .make(&codec_params, &dec_opts)
        .map_err(|e| AnalysisError::CreateDecoder {
            codec: codec_params.codec,
            source: e,
        })?;

    Ok((format_reader, decoder))
}

pub fn get_analysis(path: &PathBuf, sample_rate: u32, channels: u8) -> Result<AudioAnalysis> {
    let (mut format_reader, mut decoder) = setup_symphonia(path)?;

    if sample_rate == 0 {
        anyhow::bail!(AnalysisError::InvalidSampleRate);
    }

    if channels == 0 {
        anyhow::bail!(AnalysisError::InvalidChannelNumber);
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_WINDOW_SIZE);
    let mut fft_buffer: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); FFT_WINDOW_SIZE];
    let mut scratch_buffer: Vec<Complex<f32>> =
        vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];

    let window_coeffs_f32: Vec<f32> = hanning_iter(FFT_WINDOW_SIZE).map(|x| x as f32).collect();
    if window_coeffs_f32.len() != FFT_WINDOW_SIZE {
        anyhow::bail!(AnalysisError::HannWindowError(
            window_coeffs_f32.len(),
            FFT_WINDOW_SIZE
        ));
    }

    let mut samples_for_fft: Vec<f32> = Vec::with_capacity(FFT_WINDOW_SIZE);
    let mut spectrum_db_accumulator: Vec<f32> = vec![0.0; FFT_WINDOW_SIZE / 2];
    let mut window_count: usize = 0;
    let mut elapsed_secs = 0.0_f32;

    loop {
        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(err) => {
                return Err(AnalysisError::PacketReadError(err).into());
            }
        };

        match decoder.decode(&packet) {
            Ok(audio_buffer) => {
                let frames = audio_buffer.frames() as u64;
                if MAX_ANALYSIS_DURATION_SECONDS > 0.0 && sample_rate > 0 {
                    let dur = frames as f32 / sample_rate as f32;
                    elapsed_secs += dur;
                    if elapsed_secs >= MAX_ANALYSIS_DURATION_SECONDS {
                        break;
                    }
                }

                let spec = *audio_buffer.spec();
                let frames_usize = frames as usize;
                let chans = spec.channels.count();
                if frames_usize == 0 || chans == 0 {
                    continue;
                }

                let total_samples = frames_usize
                    .checked_mul(chans)
                    .ok_or_else(|| AnalysisError::BufferSizeOverflow(frames_usize, chans))?;
                let mut sample_buf = SampleBuffer::<f32>::new(total_samples as u64, spec);

                sample_buf.copy_interleaved_ref(audio_buffer);
                let samples_interleaved = sample_buf.samples();

                for frame in samples_interleaved.chunks_exact(channels as usize) {
                    let mono_sample: f32 = frame.iter().sum::<f32>() / channels as f32;
                    samples_for_fft.push(mono_sample);

                    if samples_for_fft.len() >= FFT_WINDOW_SIZE {
                        for i in 0..FFT_WINDOW_SIZE {
                            let sample = samples_for_fft[i];
                            fft_buffer[i] = Complex::new(sample * window_coeffs_f32[i], 0.0);
                        }
                        samples_for_fft.drain(0..FFT_WINDOW_SIZE);

                        fft.process_with_scratch(&mut fft_buffer, &mut scratch_buffer);

                        for i in 0..(FFT_WINDOW_SIZE / 2) {
                            let magnitude = fft_buffer[i].norm();
                            let magnitude_db = 20.0 * magnitude.max(1e-10).log10();
                            spectrum_db_accumulator[i] += magnitude_db;
                        }
                        window_count += 1;
                    }
                }
            }
            Err(SymphoniaError::DecodeError(_)) => {
                continue;
            }
            Err(err) => {
                return Err(AnalysisError::DecoderError(err).into());
            }
        }
    }

    let avg_spectrum_db: Vec<f32> = spectrum_db_accumulator
        .into_iter()
        .map(|sum_db| sum_db / window_count as f32)
        .collect();

    let spectral_analysis = calc_cutoff(window_count, &avg_spectrum_db, sample_rate);
    let (quality_score, overall_assessment) = calculate_quality_score(&spectral_analysis);

    Ok(AudioAnalysis {
        spectral_analysis,
        quality_score,
        overall_assessment,
    })
}

fn calculate_avg_db_in_band(
    start_hz: f32,
    end_hz: f32,
    freq_per_bin: f32,
    avg_spectrum_db: &[f32],
) -> Option<f32> {
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
    let avg_db = band.iter().sum::<f32>() / band.len() as f32;
    Some(avg_db)
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
        None => {
            return AnalysisOutcome::InconclusiveReferenceBandError;
        }
    };

    // Umbral mínimo para considerar fiable el análisis
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
                // ¡Caída significativa detectada!
                return AnalysisOutcome::CutoffDetected {
                    cutoff_frequency_hz: band_start_hz,
                    reference_level_db: reference_avg_db,
                    cutoff_band_level_db: check_avg_db,
                };
            }
        } else {
            // No se pudo calcular el dB para esta banda. Podríamos ignorarlo,
            // o registrarlo, o incluso devolver un error si es crítico.
            // Por ahora, lo ignoramos y continuamos.
            // Si fuera necesario, se podría añadir otro estado a AnalysisOutcome.
        }
    }

    AnalysisOutcome::NoCutoffDetected {
        reference_level_db: reference_avg_db,
        max_analyzed_freq_hz,
    }
}

fn calculate_quality_score(outcome: &AnalysisOutcome) -> (f32, String) {
    match outcome {
        // Caso: Se detectó un corte explícitamente.
        AnalysisOutcome::CutoffDetected {
            cutoff_frequency_hz, ..
        } => {
            // La lógica aquí es la misma que tenías en tu rama Some(cutoff_hz)
            // Usamos *cutoff_frequency_hz porque estamos haciendo match sobre una referencia (&AnalysisOutcome)
            let score = if *cutoff_frequency_hz >= 21_500.0 {
                9.8 // Muy alto, casi Nyquist para 44.1k -> Lossless o calidad extremadamente alta
            } else if *cutoff_frequency_hz >= 20_500.0 {
                9.0 // 20.5k - 21.5k -> Calidad muy alta, posible lossless con rolloff
            } else if *cutoff_frequency_hz >= 19_500.0 {
                8.0 // 19.5k - 20.5k -> Alta calidad, podría ser lossy transparente o lossless
            } else if *cutoff_frequency_hz >= 18_500.0 {
                7.0 // 18.5k - 19.5k -> Buena calidad lossy (MP3 320k típico podría estar aquí)
            } else if *cutoff_frequency_hz >= 17_500.0 {
                6.0 // 17.5k - 18.5k -> Calidad media-alta lossy
            } else if *cutoff_frequency_hz >= 16_500.0 {
                5.0 // 16.5k - 17.5k -> Calidad media lossy (MP3 ~192k o VBR q4)
            } else if *cutoff_frequency_hz >= 15_500.0 {
                4.0 // 15.5k - 16.5k -> Calidad media-baja lossy (MP3 ~128k)
            } else {
                3.0 // < 15.5k -> Calidad baja lossy
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
            };

            (score, assessment.to_string())
        }

        // Caso: No se detectó corte significativo.
        AnalysisOutcome::NoCutoffDetected { .. } => (10.0, "Perfect".to_string()),

        // Casos Inconclusos: No podemos determinar la calidad.
        AnalysisOutcome::InconclusiveNotEnoughWindows {
            processed_windows,
            required_windows,
        } => (
            0.0,
            format!(
                "Incomplete analysis (insufficient windows {}/{}). Quality not determined.",
                processed_windows, required_windows
            ),
        ),
        AnalysisOutcome::InconclusiveReferenceBandError => (
            0.0,
            "Incomplete analysis (error in reference band). Quality not determined.".to_string(),
        ),
        AnalysisOutcome::InconclusiveLowReferenceLevel { reference_level_db } => (
            0.0,
            format!(
                "Analysis inconclusive (low reference level {:.1} dB). Quality not determined.",
                reference_level_db
            ),
        ),
        _ => (0.0, format!("Analysis inconclusive")),
    }
}
