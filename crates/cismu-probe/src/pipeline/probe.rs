use std::path::Path;

use crate::{
    Analysis, Track, analysis::features, audio::AudioDecoder, error::Error, metadata::reader::MetadataReader,
    pipeline::config::ProbeConfig, prelude::FeatureFlags,
};

#[derive(Default)]
pub struct ProbeBuilder {
    cfg: ProbeConfig,
    reader: Option<Box<dyn MetadataReader + Send + Sync>>,
    decoder: Option<Box<dyn AudioDecoder + Send + Sync>>,
}

impl ProbeBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn features(mut self, flags: FeatureFlags) -> Self {
        self.cfg.features = flags;
        self
    }
    pub fn max_duration_s(mut self, secs: f32) -> Self {
        self.cfg.max_duration_s = Some(secs);
        self
    }
    pub fn prefer_embedded_pictures(mut self, yes: bool) -> Self {
        self.cfg.prefer_embedded_pictures = yes;
        self
    }
    pub fn fail_fast_on_metadata(mut self, yes: bool) -> Self {
        self.cfg.fail_fast_on_metadata = yes;
        self
    }

    pub fn with_reader<R: MetadataReader + Send + Sync + 'static>(mut self, r: R) -> Self {
        self.reader = Some(Box::new(r));
        self
    }
    pub fn with_decoder<D: AudioDecoder + Send + Sync + 'static>(mut self, d: D) -> Self {
        self.decoder = Some(Box::new(d));
        self
    }

    pub fn build(self) -> Probe {
        Probe {
            cfg: self.cfg,
            reader: self.reader.unwrap_or_else(default_reader),
            decoder: self.decoder.unwrap_or_else(default_decoder),
        }
    }
}

pub struct Probe {
    cfg: ProbeConfig,
    reader: Box<dyn MetadataReader + Send + Sync>,
    decoder: Box<dyn AudioDecoder + Send + Sync>,
}

pub fn default_reader() -> Box<dyn MetadataReader + Send + Sync> {
    #[cfg(not(feature = "lofty"))]
    {
        Box::new(crate::metadata::reader::NoopReader)
    }
}

pub fn default_decoder() -> Box<dyn AudioDecoder + Send + Sync> {
    #[cfg(feature = "ffmpeg")]
    {
        Box::new(crate::audio::decoder::FFmpegNativeDecoder::new())
    }
    #[cfg(not(feature = "ffmpeg"))]
    {
        Box::new(crate::audio::decoder::NoopDecoder)
    }
}

impl Probe {
    pub fn builder() -> ProbeBuilder {
        ProbeBuilder::default()
    }

    pub fn config(&self) -> &ProbeConfig {
        &self.cfg
    }

    pub fn config_mut(&mut self) -> &mut ProbeConfig {
        &mut self.cfg
    }

    pub fn run<P: AsRef<Path>>(&self, path: P) -> Result<ProbeResult, Error> {
        let track = self.read_metadata(&path)?;
        let features = self.analyze(path)?;
        Ok(ProbeResult { track, features })
    }

    pub fn read_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Track, Error> {
        self.reader.read(
            path.as_ref(),
            self.cfg.prefer_embedded_pictures,
            self.cfg.fail_fast_on_metadata,
        )
    }

    /// Realiza análisis musical; la función depende más de la CPU.
    pub fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<Analysis, Error> {
        let mut stream = self.decoder.open(path.as_ref())?;
        features::compute(stream.as_mut(), path.as_ref(), self.cfg.features).map_err(|e| e.into())
    }
}

impl Default for Probe {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub track: Track,
    pub features: Analysis,
}
