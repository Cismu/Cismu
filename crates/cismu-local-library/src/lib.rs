pub mod error;
pub mod extensions;
pub mod library_config;
pub mod manager;

pub use manager::{ConfigEvent, ConfigManager};

pub struct LocalLibrary {
    pub config: ConfigManager,
}

impl LocalLibrary {
    pub fn new(config: ConfigManager) -> Self {
        LocalLibrary { config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
