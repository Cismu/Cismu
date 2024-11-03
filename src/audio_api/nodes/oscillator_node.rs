use super::AudioNode;

// OscillatorNode for generating waveforms
pub struct OscillatorNode {
    frequency: f32,
}

impl OscillatorNode {
    pub fn new() -> Self {
        OscillatorNode { frequency: 440.0 }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl AudioNode for OscillatorNode {
    fn connect(&self, destination: &dyn AudioNode) {
        // Connect oscillator to another node
    }
}
