use std::error::Error;
use std::f32;

// Representa las opciones de configuración para AudioBuffer
pub struct AudioBufferOptions {
    pub number_of_channels: u32,
    pub length: u32,
    pub sample_rate: f32,
}

// Estructura que representa el AudioBuffer
pub struct AudioBuffer {
    sample_rate: f32,
    length: u32,
    number_of_channels: u32,
    internal_data: Vec<Vec<f32>>, // Almacena datos de cada canal en una Vec separada
}

impl AudioBuffer {
    // Constructor para crear un nuevo AudioBuffer con opciones especificadas
    pub fn new(options: AudioBufferOptions) -> Result<Self, Box<dyn Error>> {
        if options.sample_rate <= 0.0 || options.length == 0 || options.number_of_channels == 0 {
            return Err("NotSupportedError: Valores fuera de rango".into());
        }

        let internal_data =
            vec![vec![0.0; options.length as usize]; options.number_of_channels as usize];

        Ok(Self {
            sample_rate: options.sample_rate,
            length: options.length,
            number_of_channels: options.number_of_channels,
            internal_data,
        })
    }

    pub fn from_wav(file_path: &str) -> Result<Self, Box<dyn Error>> {
        // Abre el archivo WAV
        let mut reader = hound::WavReader::open(file_path)?;
        let spec = reader.spec();

        // Verifica que el formato sea PCM en punto flotante o entero
        if spec.sample_format != hound::SampleFormat::Float && spec.bits_per_sample != 16 {
            return Err(
                "Formato no soportado: se espera PCM de 16 bits o de punto flotante".into(),
            );
        }

        // Configura los parámetros del buffer
        let number_of_channels = spec.channels as u32;
        let sample_rate = spec.sample_rate as f32;
        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32) // Normaliza los datos de 16 bits a rango [-1.0, 1.0]
            .collect();

        let length = samples.len() as u32 / number_of_channels;

        // Crear el buffer y organizar los datos en canales
        let mut internal_data = vec![vec![0.0; length as usize]; number_of_channels as usize];
        for (i, sample) in samples.iter().enumerate() {
            let channel = (i % number_of_channels as usize) as usize;
            let frame = (i / number_of_channels as usize) as usize;
            internal_data[channel][frame] = *sample;
        }

        Ok(Self {
            number_of_channels,
            length,
            sample_rate,
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
        let num_frames_to_copy = destination
            .len()
            .min(channel_data.len().saturating_sub(buffer_offset));

        destination[..num_frames_to_copy]
            .copy_from_slice(&channel_data[buffer_offset..buffer_offset + num_frames_to_copy]);
        Ok(())
    }

    // Copia datos desde un array fuente a un canal específico
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
        let num_frames_to_copy = source
            .len()
            .min(channel_data.len().saturating_sub(buffer_offset));

        channel_data[buffer_offset..buffer_offset + num_frames_to_copy]
            .copy_from_slice(&source[..num_frames_to_copy]);
        Ok(())
    }
}
