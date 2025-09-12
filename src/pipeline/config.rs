use crate::prelude::FeatureFlags;

#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub features: FeatureFlags,      // qué calcular (RMS, centroid, MFCC, tempo…)
    pub max_duration_s: Option<f32>, // cortar análisis si el archivo es largo
    pub prefer_embedded_pictures: bool,
    pub fail_fast_on_metadata: bool,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            features: FeatureFlags::default_all(),
            max_duration_s: None,
            prefer_embedded_pictures: true,
            fail_fast_on_metadata: false,
        }
    }
}
