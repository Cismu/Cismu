use std::error::Error;
use std::f32;

pub struct AudioBufferOptions {
    pub number_of_channels: u32,
    pub length: u32,
    pub sample_rate: f32,
}

pub struct AudioBuffer {
    sample_rate: f32,
    length: u32,
    number_of_channels: u32,
    internal_data: Vec<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new(options: AudioBufferOptions) -> Result<Self, Box<dyn Error>> {
        if options.sample_rate <= 0.0 || options.length == 0 || options.number_of_channels == 0 {
            return Err("NotSupportedError: Valores fuera de rango".into());
        }

        let internal_data = vec![vec![0.0; options.length as usize]; options.number_of_channels as usize];

        Ok(Self {
            sample_rate: options.sample_rate,
            length: options.length,
            number_of_channels: options.number_of_channels,
            internal_data,
        })
    }

    // Retorna la tasa de muestreo
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    // Retorna la longitud del buffer en sample-frames
    pub fn length(&self) -> u32 {
        self.length
    }

    // Retorna la duración del buffer en segundos
    pub fn duration(&self) -> f64 {
        self.length as f64 / self.sample_rate as f64
    }

    // Retorna el número de canales
    pub fn number_of_channels(&self) -> u32 {
        self.number_of_channels
    }

    // Obtiene una referencia mutable a los datos de un canal específico como un slice mutable de f32
    pub fn get_channel_data(&mut self, channel: u32) -> Result<&mut [f32], Box<dyn Error>> {
        if channel >= self.number_of_channels {
            return Err("IndexSizeError: Número de canal fuera de rango".into());
        }
        Ok(&mut self.internal_data[channel as usize])
    }

    // Copia datos de un canal a una array de destino
    pub fn copy_from_channel(
        &self,
        destination: &mut [f32],
        channel: u32,
        buffer_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        if channel >= self.number_of_channels {
            return Err("IndexSizeError: Número de canal fuera de rango".into());
        }

        let channel_data = &self.internal_data[channel as usize];
        let num_frames_to_copy = destination.len().min(channel_data.len().saturating_sub(buffer_offset));

        destination[..num_frames_to_copy]
            .copy_from_slice(&channel_data[buffer_offset..buffer_offset + num_frames_to_copy]);
        Ok(())
    }

    pub fn copy_to_channel(
        &mut self,
        source: &[f32],
        channel: u32,
        buffer_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        if channel >= self.number_of_channels {
            return Err("IndexSizeError: Número de canal fuera de rango".into());
        }

        let channel_data = &mut self.internal_data[channel as usize];
        let num_frames_to_copy = source.len().min(channel_data.len().saturating_sub(buffer_offset));

        channel_data[buffer_offset..buffer_offset + num_frames_to_copy].copy_from_slice(&source[..num_frames_to_copy]);
        Ok(())
    }
}
