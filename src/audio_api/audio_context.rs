use std::sync::{Arc, Mutex};

use super::{
    audio_destination_node::AudioDestinationNode, nodes::AudioBufferSourceNode, AudioBuffer,
    AudioBufferOptions,
};

#[derive(Debug, PartialEq)]
pub enum AudioContextState {
    Suspended,
    Running,
    Closed,
}

pub struct AudioContext {
    state: AudioContextState,
    sample_rate: f32,
    current_time: f64,
    pub destination: AudioDestinationNode,
}

impl AudioContext {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: AudioContextState::Suspended,
            sample_rate,
            current_time: 0.0,
            destination: AudioDestinationNode::new(2, sample_rate),
        }
    }

    pub fn state(&self) -> &AudioContextState {
        &self.state
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn resume(&mut self) {
        if self.state == AudioContextState::Suspended {
            self.state = AudioContextState::Running;
            self.destination.initialize_output_stream();
        }
    }

    pub fn suspend(&mut self) {
        if self.state == AudioContextState::Running {
            self.state = AudioContextState::Suspended;
        }
    }

    pub fn process_audio(&mut self, duration: f64) {
        if self.state == AudioContextState::Running {
            self.current_time += duration;
        }
    }

    /// Conecta un AudioBufferSourceNode al nodo de destino (AudioDestinationNode)
    pub fn connect(&mut self, source: Arc<Mutex<AudioBufferSourceNode>>) {
        self.destination.add_source(source);
    }

    // Crea un buffer con la configuración especificada
    pub fn create_buffer(
        &self,
        number_of_channels: u32,
        length: u32,
        sample_rate: f32,
    ) -> Arc<Mutex<AudioBuffer>> {
        let options = AudioBufferOptions {
            number_of_channels,
            length,
            sample_rate,
        };
        Arc::new(Mutex::new(
            AudioBuffer::new(options).expect("Error creando el buffer"),
        ))
    }

    // Crea y configura un AudioBufferSourceNode
    pub fn create_buffer_source(&self, buffer: Arc<Mutex<AudioBuffer>>) -> AudioBufferSourceNode {
        let mut source_node = AudioBufferSourceNode::new();
        source_node.set_buffer(buffer).unwrap();
        source_node
    }
}
