mod gain_node;
mod oscillator_node;
mod audio_buffer_source_node;

pub use gain_node::GainNode;
pub use oscillator_node::OscillatorNode;
pub use audio_buffer_source_node::AudioBufferSourceNode;

// A basic AudioNode trait that different node types will implement
pub trait AudioNode {
    fn connect(&self, destination: &dyn AudioNode);
}
