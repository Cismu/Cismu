bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FeatureFlags: u32 {
        const RMS              = 1 << 0;
        const SPECTRAL_CENTROID= 1 << 1;
        const MFCC             = 1 << 2;
        const TEMPO            = 1 << 3;
        const CHROMA           = 1 << 4;
        const ALL = Self::RMS.bits()
                 | Self::SPECTRAL_CENTROID.bits()
                 | Self::MFCC.bits()
                 | Self::TEMPO.bits()
                 | Self::CHROMA.bits();
    }
}

impl FeatureFlags {
    pub fn default_all() -> Self {
        FeatureFlags::ALL
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    pub rms: Option<f32>,
    pub spectral_centroid_hz: Option<f32>,
    pub tempo_bpm: Option<f32>,
    // etc… agrega tus campos / vectores
}
