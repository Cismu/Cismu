use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::nodes::AudioBufferSourceNode;

#[derive(Debug)]
pub enum ChannelCountMode {
    Max,
    ClampedMax,
    Explicit,
}

#[derive(Debug)]
pub enum ChannelInterpretation {
    Speakers,
    Discrete,
}

pub struct AudioDestinationNode {
    active_sources: Vec<Arc<Mutex<AudioBufferSourceNode>>>,
    max_channel_count: u32,
    channel_count: u32,
    channel_count_mode: ChannelCountMode,
    channel_interpretation: ChannelInterpretation,
    sample_rate: f32,
    stream: Option<Stream>,
}

impl AudioDestinationNode {
    pub fn new(max_channel_count: u32, sample_rate: f32) -> Self {
        let destination = Self {
            active_sources: Vec::new(),
            max_channel_count,
            channel_count: 2, // Por defecto, 2 canales para salida estéreo
            channel_count_mode: ChannelCountMode::Explicit,
            channel_interpretation: ChannelInterpretation::Speakers,
            sample_rate,
            stream: None,
        };

        destination
    }

    pub fn initialize_output_stream(&mut self) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No se encontró un dispositivo de salida");

        let supported_config = device.default_output_config().unwrap();
        let sample_format = supported_config.sample_format();
        let sample_rate = supported_config.sample_rate().0 as f32;
        let channels = supported_config.channels();

        // Crear y configurar el stream basado en el formato de muestra
        let stream = match sample_format {
            SampleFormat::F32 => self.build_stream::<f32>(&device, sample_rate, channels),
            _ => panic!("Formato de muestra no soportado"),
        };

        self.stream = Some(stream);
    }

    fn build_stream<T>(&self, device: &cpal::Device, sample_rate: f32, channels: u16) -> Stream
    where
        T: cpal::Sample + From<f32> + cpal::SizedSample,
    {
        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        };

        let active_sources = self.active_sources.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    Self::write_data(data, &active_sources, sample_rate);
                },
                |err| eprintln!("Error en el stream de salida: {:?}", err),
                None, // Especifica None para el intervalo de procesamiento
            )
            .expect("No se pudo crear el stream de salida");

        stream.play().expect("No se pudo reproducir el stream");
        stream
    }

    fn write_data<T>(
        output: &mut [T],
        sources: &[Arc<Mutex<AudioBufferSourceNode>>],
        sample_rate: f32,
    ) where
        T: cpal::Sample + From<f32>,
    {
        let num_frames = output.len() / 2; // Número de cuadros para estéreo

        // Mezclar todas las fuentes activas y generar el audio final
        let mut buffer = vec![0.0; num_frames * 2]; // Estéreo

        for source in sources {
            let mut source = source.lock().unwrap();
            let samples = source.process(num_frames, 1.0 / sample_rate as f64);

            for (i, &sample) in samples.iter().enumerate() {
                buffer[i * 2] += sample; // Canal izquierdo
                buffer[i * 2 + 1] += sample; // Canal derecho
            }
        }

        // Convertir los datos a T y escribir en el buffer de salida
        for (i, sample) in buffer.iter().enumerate() {
            output[i] = T::from(*sample);
        }
    }

    /// Agrega una fuente de audio activa al nodo de destino
    pub fn add_source(&mut self, source: Arc<Mutex<AudioBufferSourceNode>>) {
        self.active_sources.push(source);
    }

    /// Retorna el número máximo de canales soportado
    pub fn max_channel_count(&self) -> u32 {
        self.max_channel_count
    }

    /// Retorna el número de canales actual
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    /// Cambia el número de canales, si está dentro de los límites
    pub fn set_channel_count(&mut self, channel_count: u32) -> Result<(), String> {
        if channel_count <= self.max_channel_count {
            self.channel_count = channel_count;
            Ok(())
        } else {
            Err("IndexSizeError: El número de canales está fuera del rango permitido.".to_string())
        }
    }

    /// Retorna el modo de conteo de canales actual
    pub fn channel_count_mode(&self) -> &ChannelCountMode {
        &self.channel_count_mode
    }

    /// Retorna la interpretación de canales actual
    pub fn channel_interpretation(&self) -> &ChannelInterpretation {
        &self.channel_interpretation
    }
}
