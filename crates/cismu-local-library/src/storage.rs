use std::{collections::HashSet, path::PathBuf, sync::Arc};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::RngCore;

pub struct LocalStorage {
    config: Arc<LocalStorageConfig>,
}

impl LocalStorage {
    pub fn new(config: Arc<LocalStorageConfig>) -> Self {
        LocalStorage { config }
    }
}

#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Sqlite(PathBuf),
}

#[derive(Debug, Clone)]
pub struct LocalStorageConfig {
    pub database: DatabaseConfig,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        LocalStorageConfig {
            database: DatabaseConfig::Sqlite(PathBuf::from("/tmp/database.db")),
        }
    }
}

fn generate_id(num_bytes: usize) -> String {
    let mut random_bytes = vec![0u8; num_bytes];
    rand::rng().fill_bytes(&mut random_bytes);
    BASE64_URL_SAFE_NO_PAD.encode(&random_bytes)
}

fn generate_unique_id(existing_ids: &mut HashSet<String>) -> String {
    let mut retries = 0;
    let mut size = 4;

    loop {
        let new_id = generate_id(size);

        if existing_ids.insert(new_id.clone()) {
            return new_id;
        }

        retries += 1;

        if retries >= 10 {
            retries = 0;
            size += 1;
        }
    }
}
