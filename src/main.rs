mod application;
mod audio_api;
mod renderer;

use crate::application::Application;
use audio_api::{AudioBuffer, AudioContext};
use hound::WavReader;
use rand::Rng;
use std::sync::{Arc, Mutex};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa el loop de eventos y el contexto de audio
    let event_loop = EventLoop::new()?;
    let audio_context = AudioContext::new(None, None);

    // Crea y rellena un buffer de ruido
    let buffer = initialize_audio_buffer(&audio_context)?;
    display_buffer_info(&buffer);

    // Resamplea el buffer si es necesario
    resample_buffer_if_needed(&buffer, &audio_context)?;

    // Carga y muestra la información de un archivo WAV
    let wav_buffer = load_wav_to_buffer("windows_background.wav", &audio_context)?;

    display_buffer_info(&wav_buffer);

    // Inicializa y ejecuta la aplicación
    let mut app = Application::new(&event_loop)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn initialize_audio_buffer(
    audio_context: &AudioContext,
) -> Result<Arc<Mutex<AudioBuffer>>, Box<dyn std::error::Error>> {
    let buffer = audio_context.create_buffer(1, 22050, 22050.0);

    {
        let mut buffer_locked = buffer.lock().unwrap();
        if buffer_locked.number_of_channels() == 2 {
            println!("El buffer es estéreo.");
        }

        fill_audio_buffer_with_noise(&mut buffer_locked)?;
    }

    Ok(buffer)
}

fn fill_audio_buffer_with_noise(buffer: &mut AudioBuffer) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    for channel in 0..buffer.number_of_channels() {
        let now_buffering = buffer.get_channel_data(channel)?;
        for sample in now_buffering.iter_mut() {
            *sample = rng.gen_range(-1.0..1.0);
        }
    }
    Ok(())
}

fn display_buffer_info(buffer: &Arc<Mutex<AudioBuffer>>) {
    let mut buffer_locked = buffer.lock().unwrap();
    println!("Tasa de muestreo: {}", buffer_locked.sample_rate());
    println!("Duración del buffer: {}", buffer_locked.duration());
    println!("Número de canales: {}", buffer_locked.number_of_channels());

    let channel_data = buffer_locked
        .get_channel_data(0)
        .expect("Error al obtener datos del canal");
    for i in 0..10 {
        print!("{:.2} ", channel_data[i]);
    }
    println!("...");
}

fn resample_buffer_if_needed(
    buffer: &Arc<Mutex<AudioBuffer>>,
    audio_context: &AudioContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer_locked = buffer.lock().unwrap();
    let buffer_sample_rate = buffer_locked.sample_rate();

    if buffer_sample_rate != audio_context.sample_rate {
        println!(
            "Resampleando el buffer de {}Hz a {}Hz",
            buffer_sample_rate, audio_context.sample_rate
        );

        // Realizamos el resampling y obtenemos el nuevo buffer sin el Arc<Mutex> (ya que la función devuelve un AudioBuffer)
        let mut resampled_buffer = audio_context.resample_buffer(&mut buffer_locked)?;

        println!("Reproduciendo el buffer resampleado a {}Hz", audio_context.sample_rate);
        display_buffer_info_raw(&mut resampled_buffer);
    } else {
        println!("Reproduciendo el buffer a {}Hz", audio_context.sample_rate);
    }

    Ok(())
}

// Función auxiliar para mostrar información de un AudioBuffer sin Arc<Mutex>
fn display_buffer_info_raw(buffer: &mut AudioBuffer) {
    println!("Tasa de muestreo: {}", buffer.sample_rate());
    println!("Duración del buffer: {}", buffer.duration());
    println!("Número de canales: {}", buffer.number_of_channels());

    let channel_data = buffer.get_channel_data(0).expect("Error al obtener datos del canal");
    for i in 0..10 {
        print!("{:.2} ", channel_data[i]);
    }
    println!("...");
}

fn load_wav_to_buffer(
    file_path: &str,
    audio_context: &AudioContext,
) -> Result<Arc<Mutex<AudioBuffer>>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(file_path)?;
    let spec = reader.spec();

    // Verificación de formato
    if spec.sample_format != hound::SampleFormat::Float && spec.bits_per_sample != 16 {
        return Err("Formato no soportado: se espera PCM de 16 bits o de punto flotante".into());
    }

    // Cargar datos y crear el buffer
    let number_of_channels = spec.channels as u32;
    let sample_rate = spec.sample_rate as f32;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let length = samples.len() as u32 / number_of_channels;
    let internal_data = organize_samples_into_channels(samples, number_of_channels, length);
    let wav_buffer = audio_context.create_buffer_from_data(internal_data, sample_rate);

    Ok(wav_buffer)
}

fn organize_samples_into_channels(samples: Vec<f32>, channels: u32, length: u32) -> Vec<Vec<f32>> {
    let mut channel_data = vec![vec![0.0; length as usize]; channels as usize];
    for (i, sample) in samples.iter().enumerate() {
        let channel = i % channels as usize;
        let frame = i / channels as usize;
        channel_data[channel][frame] = *sample;
    }
    channel_data
}
