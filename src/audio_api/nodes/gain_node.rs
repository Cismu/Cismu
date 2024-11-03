use super::AudioNode;

// GainNode to control volume
pub struct GainNode {
    gain: f32,
}

impl GainNode {
    pub fn new() -> Self {
        GainNode { gain: 1.0 }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}
impl AudioNode for GainNode {
    fn connect(&self, destination: &dyn AudioNode) {
        // Connect gain node to another node
    }
}
