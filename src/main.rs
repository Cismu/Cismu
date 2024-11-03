mod application;
mod audio_api;
mod renderer;

use rand::Rng;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::application::Application;
use audio_api::{AudioBuffer, AudioContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Crear un buffer estéreo de tres segundos con una tasa de muestreo de 44100 Hz
    let audio_context = AudioContext::new(None, None);
    let my_array_buffer = audio_context.create_buffer(1, 22050, 22050.0);

    // Verificar si el buffer es estéreo
    let mut buffer = my_array_buffer.lock().unwrap();
    if buffer.number_of_channels() == 2 {
        println!("El buffer es estéreo.");
    }

    fill_audio_buffer_with_noise(&mut buffer)?;

    println!("Tasa de muestreo: {}", buffer.sample_rate());
    println!("Duración del buffer: {}", buffer.duration());
    println!("Número de canales: {}", buffer.number_of_channels());

    let channel_data = buffer.get_channel_data(0)?;
    for i in 0..10 {
        print!("{:.2} ", channel_data[i]);
    }
    println!("...");

    // Verificar si necesitamos resamplear
    if buffer.sample_rate() != audio_context.sample_rate {
        println!(
            "Resampleando el buffer de {}Hz a {}Hz",
            buffer.sample_rate(),
            audio_context.sample_rate
        );
        let resampled_buffer = audio_context.resample_buffer(&mut buffer)?;
        println!("Reproduciendo el buffer resampleado a {}Hz", audio_context.sample_rate);

        println!("Duración del buffer resampleado: {}", resampled_buffer.duration());
        println!("Número de canales: {}", resampled_buffer.number_of_channels());
        println!("Longitud del buffer resampleado: {}", resampled_buffer.length());
    } else {
        println!("Reproduciendo el buffer a {}Hz", audio_context.sample_rate);
    }

    let mut reader = hound::WavReader::open("windows_background.wav")?;
    let spec = reader.spec();

    // Verifica que el formato sea PCM en punto flotante o entero
    if spec.sample_format != hound::SampleFormat::Float && spec.bits_per_sample != 16 {
        return Err("Formato no soportado: se espera PCM de 16 bits o de punto flotante".into());
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

    let my_wav_buffer = audio_context.create_buffer_from_data(internal_data, sample_rate);
    let my_wav_buffer = my_wav_buffer.lock().unwrap();
    println!("Reproduciendo el buffer de un archivo WAV a {}Hz", sample_rate);
    println!("Duración del buffer: {}", my_wav_buffer.duration());
    println!("Número de canales: {}", my_wav_buffer.number_of_channels());
    println!("Longitud del buffer: {}", my_wav_buffer.length());

    // #################################################################################

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Application::new(&event_loop)?;

    event_loop.run_app(&mut app)?;

    Ok(())
}

fn fill_audio_buffer_with_noise(buffer: &mut AudioBuffer) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng(); // Crea un generador de números aleatorios

    // Itera sobre cada canal
    for channel in 0..buffer.number_of_channels() {
        let now_buffering = buffer.get_channel_data(channel)?;

        // Llenar el canal con valores aleatorios entre -1.0 y 1.0
        for sample in now_buffering.iter_mut() {
            *sample = rng.gen_range(-1.0..1.0);
        }
    }

    Ok(())
}
