use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("The duration of the file is less than the minimum allowed.")]
    DurationTooShort,
}
