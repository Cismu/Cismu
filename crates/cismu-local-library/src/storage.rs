use cismu_paths::PATHS;
use rusqlite::{Connection, Result, params};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::info;

/// Configuración de base de datos
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

pub struct LocalStorage {
    conn: Arc<Mutex<Connection>>,
    _config: Arc<LocalStorageConfig>,
}

impl LocalStorage {
    pub fn new(config: LocalStorageConfig) -> Result<Self> {
        let conn = match &config.database {
            DatabaseConfig::Sqlite(path) => {
                info!("Opening database connection at {}", path.display());
                Connection::open(path)?
            }
        };

        let conn = Arc::new(Mutex::new(conn));

        {
            let guard = conn.lock().unwrap();
            guard.pragma_update(None, "foreign_keys", &"ON")?;
        }

        let storage = LocalStorage {
            conn,
            _config: Arc::new(config),
        };

        // 3) Inicializar esquema (CREATE TABLE…)
        storage.init_schema()?;

        Ok(storage)
    }

    /// Ejecuta en bloque la DDL para crear todas las tablas
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            BEGIN;

            CREATE TABLE IF NOT EXISTS artists (
              id          INTEGER PRIMARY KEY NOT NULL,
              name        TEXT    NOT NULL,
              bio         TEXT,
              created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS artist_sites (
              artist_id  INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
              url        TEXT    NOT NULL,
              PRIMARY KEY (artist_id, url)
            );

            CREATE TABLE IF NOT EXISTS artist_variations (
              artist_id  INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
              variation  TEXT    NOT NULL,
              PRIMARY KEY (artist_id, variation)
            );

            CREATE TABLE IF NOT EXISTS genres (
              id    INTEGER PRIMARY KEY NOT NULL,
              name  TEXT    NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS artists_genres (
              artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
              genre_id  INTEGER NOT NULL REFERENCES genres(id)  ON DELETE CASCADE,
              PRIMARY KEY (artist_id, genre_id)
            );

            CREATE TABLE IF NOT EXISTS albums (
              id           INTEGER PRIMARY KEY NOT NULL,
              title        TEXT    NOT NULL,
              release_date DATETIME,
              notes        TEXT,
              created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS styles (
              id    INTEGER PRIMARY KEY NOT NULL,
              name  TEXT    NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS album_artists (
              album_id  INTEGER NOT NULL REFERENCES albums(id)  ON DELETE CASCADE,
              artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
              PRIMARY KEY (album_id, artist_id)
            );

            CREATE TABLE IF NOT EXISTS album_genres (
              album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
              genre_id INTEGER NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
              PRIMARY KEY (album_id, genre_id)
            );

            CREATE TABLE IF NOT EXISTS album_styles (
              album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
              style_id INTEGER NOT NULL REFERENCES styles(id) ON DELETE CASCADE,
              PRIMARY KEY (album_id, style_id)
            );

            COMMIT;
        "#,
        )?;
        Ok(())
    }

    /// Inserta un artista y devuelve su `id`
    pub fn insert_artist(&mut self, name: &str, bio: Option<&str>) -> Result<i64> {
        let mut conn = self.conn.lock().unwrap();

        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO artists (name, bio) VALUES (?1, ?2)",
            params![name, bio],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    /// Lee un artista por su `id`
    pub fn get_artist(&self, artist_id: i64) -> Result<Option<(String, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name, bio FROM artists WHERE id = ?1")?;
        let mut rows = stmt.query(params![artist_id])?;
        if let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let bio: Option<String> = row.get(1)?;
            Ok(Some((name, bio)))
        } else {
            Ok(None)
        }
    }
}
