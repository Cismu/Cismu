mod application;
mod audio_api;
mod renderer;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::application::Application;
use winit::event_loop::{ControlFlow, EventLoop};

use audio_api::{AudioBuffer, AudioContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Cargar un archivo WAV en el AudioBuffer
    let buffer = AudioBuffer::from_wav("windows_background.wav")?;
    let mut audio_context = AudioContext::new(buffer.sample_rate());

    let buffer = Arc::new(Mutex::new(buffer));
    let mut source_node = audio_context.create_buffer_source(buffer);

    // source_node.get_next_samples();
    println!("Buffer source creado con éxito");

    source_node.connect(&mut audio_context.destination);
    source_node.set_loop(true);
    audio_context.resume();

    thread::sleep(Duration::from_secs(5));

    return Ok(());

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Application::new(&event_loop)?;

    event_loop.run_app(&mut app)?;

    Ok(())
}
