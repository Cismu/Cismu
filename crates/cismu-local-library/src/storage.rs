use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cismu_core::discography::{UnresolvedTrack, song::Song};
use rusqlite::Connection;
use tracing::info;

use cismu_paths::PATHS;

use crate::embedded::migrations::runner;

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
            database: DatabaseConfig::Sqlite(PATHS.library_db.clone()),
        }
    }
}

#[derive(Debug)]
pub struct LocalStorage {
    conn: Arc<Mutex<Connection>>,
}

impl LocalStorage {
    pub fn new(config: LocalStorageConfig) -> Result<Self> {
        let mut conn = match &config.database {
            DatabaseConfig::Sqlite(path) => {
                info!("Abriendo conexión con la base de datos en {}", path.display());
                Connection::open(path)?
            }
        };

        Self::initialize_connection(&mut conn)?;

        Ok(LocalStorage {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn initialize_connection(conn: &mut Connection) -> Result<()> {
        // Habilitar `foreign keys` es una buena práctica para mantener la integridad de los datos.
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Habilita el modo "Write-Ahead Logging" para un mejor rendimiento y concurrencia.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Establece un nivel de sincronización que ofrece buena seguridad sin sacrificar rendimiento.
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        // Espera hasta 10 segundos (10000 ms) si la base de datos está ocupada antes de fallar.
        conn.pragma_update(None, "busy_timeout", 10000)?;

        info!("Ejecutando migraciones de la base de datos...");

        let migrations = runner();
        migrations.run(conn)?;

        info!("Migraciones completadas exitosamente.");
        Ok(())
    }
}

impl LocalStorage {
    pub fn resolve_unresolved_track(&self, track: UnresolvedTrack) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let song = Song::default();

        Ok(())
    }
}
