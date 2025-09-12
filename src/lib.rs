pub mod error;
pub mod prelude;

pub mod analysis;
pub mod audio;
pub mod metadata;
pub mod pipeline;

pub use analysis::features::FeatureSet;
pub use metadata::model::{Tag, Track};
pub use pipeline::probe::Probe;
