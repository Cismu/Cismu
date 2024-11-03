use std::sync::{Arc, Mutex};

use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

use super::{audio_destination_node::AudioDestinationNode, AudioBuffer, AudioBufferOptions};

pub struct AudioContext {
    pub sample_rate: f32,
    pub base_latency: f32,
    pub output_latency: f32,
    render_quantum_size: u8,
    latency_hint: AudioContextLatencyCategory,
    pub destination: AudioDestinationNode,
}

impl AudioContext {
    pub fn new(sample_rate: Option<f32>, latency_hint: Option<AudioContextLatencyCategory>) -> Self {
        let sample_rate = sample_rate.unwrap_or(44100.0); // Default to 44.1 kHz
        let latency_hint = latency_hint.unwrap_or(AudioContextLatencyCategory::Interactive);

        // Calculate base latency in seconds
        let render_quantum_size = 128u8;
        let base_latency = (2 * render_quantum_size as u16) as f32 / sample_rate;

        // Create the destination node and fetch output latency
        let destination = AudioDestinationNode::new(sample_rate);
        let output_latency = destination.output_latency();

        Self {
            sample_rate,
            base_latency,
            output_latency,
            render_quantum_size,
            latency_hint,
            destination,
        }
    }

    pub fn create_buffer(&self, number_of_channels: u32, length: u32, sample_rate: f32) -> Arc<Mutex<AudioBuffer>> {
        let options = AudioBufferOptions {
            number_of_channels,
            length,
            sample_rate,
        };
        Arc::new(Mutex::new(AudioBuffer::new(options).expect("Error creating buffer")))
    }

    pub fn create_buffer_from_data(&self, data: Vec<Vec<f32>>, sample_rate: f32) -> Arc<Mutex<AudioBuffer>> {
        let number_of_channels = data.len() as u32;
        let length = data[0].len() as u32;
        let options = AudioBufferOptions {
            number_of_channels,
            length,
            sample_rate,
        };
        let mut buffer = AudioBuffer::new(options).expect("Error creating buffer");

        for (channel, channel_data) in data.iter().enumerate() {
            buffer
                .copy_to_channel(channel_data, channel as u32, 0)
                .expect("Error copying data to buffer");
        }

        Arc::new(Mutex::new(buffer))
    }

    pub fn resample_buffer(&self, buffer: &mut AudioBuffer) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
        let channels = buffer.number_of_channels() as usize;
        let input_sample_rate = buffer.sample_rate();
        let resample_ratio = self.sample_rate as f64 / input_sample_rate as f64;

        // Configurar los parámetros de interpolación de Rubato
        let params = SincInterpolationParameters {
            sinc_len: 256,                                // Longitud del filtro sinc para una calidad alta
            f_cutoff: 0.95,                               // Frecuencia de corte
            interpolation: SincInterpolationType::Linear, // Interpolación lineal
            oversampling_factor: 256,                     // Factor de sobremuestreo
            window: WindowFunction::BlackmanHarris2,      // Ventana de Blackman-Harris
        };

        // Crear el resampler de Rubato
        let mut resampler = SincFixedIn::<f64>::new(
            resample_ratio,           // Ratio de resampling calculado
            2.0,                      // Máximo ratio de resampling relativo
            params,                   // Parámetros de interpolación
            buffer.length() as usize, // Tamaño de chunk (número de frames en la entrada)
            channels,                 // Número de canales
        )?;

        // Organizar los datos del buffer en una estructura compatible con Rubato
        let input_data: Vec<Vec<f64>> = (0..channels)
            .map(|channel| {
                buffer
                    .get_channel_data(channel as u32)
                    .unwrap()
                    .iter()
                    .map(|&s| s as f64)
                    .collect()
            })
            .collect();

        // Ejecutar el resampling
        let output_data = resampler.process(&input_data, None)?;

        // Crear un nuevo buffer para almacenar los datos resampleados
        let new_length = ((buffer.length() as f64) * resample_ratio).round() as u32;

        let mut resampled_buffer = AudioBuffer::new(AudioBufferOptions {
            number_of_channels: channels as u32,
            length: new_length,
            sample_rate: self.sample_rate,
        })?;

        // Copiar los datos resampleados al nuevo buffer
        for (channel, data) in output_data.iter().enumerate() {
            let data_f32: Vec<f32> = data.iter().map(|&s| s as f32).collect();
            resampled_buffer.copy_to_channel(&data_f32, channel as u32, 0)?;
        }

        Ok(resampled_buffer)
    }
}

#[derive(PartialEq, Debug)]
pub enum AudioContextLatencyCategory {
    Balanced,    // Balances latency and power consumption.
    Interactive, // Minimizes latency; may increase power consumption.
    Playback,    // Prioritizes uninterrupted playback with lowest power usage.
}

#[derive(PartialEq, Debug)]
pub enum AudioContextState {
    Suspended,
    Running,
    Closed,
}
