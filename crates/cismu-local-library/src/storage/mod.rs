mod embedded;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cismu_core::discography::{
    UnresolvedTrack, artist::ArtistId, release::ReleaseId, release_track::ReleaseTrackId, song::SongId,
};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::{info, trace};

use cismu_paths::PATHS;

use embedded::migrations::runner;

use crate::enrichment::acoustid::AcoustidResult;

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
        conn.pragma_update(None, "busy_timeout", 5000)?;

        info!("Ejecutando migraciones de la base de datos...");

        let migrations = runner();
        let migrations = migrations.run(conn)?;
        let applied_migrations = migrations.applied_migrations();

        for migration in applied_migrations {
            trace!("Migración aplicada: {:?}", migration);
        }

        info!("Migraciones completadas exitosamente.");
        Ok(())
    }
}

impl LocalStorage {
    pub fn resolve_unresolved_track(&self, track: UnresolvedTrack) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let performer_ids = Self::find_or_create_artists(&tx, &track.performers)?;
        let featured_artist_ids = Self::find_or_create_artists(&tx, &track.featured_artists)?;
        let composer_ids = Self::find_or_create_artists(&tx, &track.composers)?;
        let producer_ids = Self::find_or_create_artists(&tx, &track.producers)?;
        let album_artist_ids = Self::find_or_create_artists(&tx, &track.album_artists)?;

        let release_id = Self::find_or_create_release(
            &tx,
            track.album.as_deref().unwrap_or("Unknown Album"),
            &album_artist_ids,
        )?;

        let song_id = Self::find_or_create_song(
            &tx,
            track.title.as_deref().unwrap_or("Unknown Title"),
            &performer_ids,
            &featured_artist_ids,
            &composer_ids,
            &producer_ids,
        )?;

        let release_track_id = Self::create_release_track(&tx, song_id, release_id, &track)?;

        tx.execute(
            "INSERT OR IGNORE INTO fingerprint_queue (release_track_id) VALUES (?1)",
            params![release_track_id],
        )?;

        tx.commit()?;

        info!("Pista resuelta y guardada: {}", track.path.display());

        Ok(())
    }

    /// Busca artistas por nombre; si no existen, los crea. Devuelve sus IDs.
    fn find_or_create_artists(
        tx: &rusqlite::Transaction,
        artist_names: &[String],
    ) -> Result<Vec<ArtistId>> {
        let mut artist_ids = Vec::new();
        let mut stmt_select = tx.prepare("SELECT id FROM artists WHERE name = ?1")?;
        let mut stmt_insert = tx.prepare("INSERT INTO artists (name) VALUES (?1)")?;

        for name in artist_names.iter().filter(|n| !n.is_empty()) {
            if let Some(id) = stmt_select.query_row([name], |row| row.get(0)).optional()? {
                artist_ids.push(id);
            } else {
                stmt_insert.execute([name])?;
                artist_ids.push(tx.last_insert_rowid() as ArtistId);
            }
        }
        Ok(artist_ids)
    }

    /// Busca un release por título y artistas principales; si no existe, lo crea.
    fn find_or_create_release(
        tx: &rusqlite::Transaction,
        title: &str,
        album_artist_ids: &[ArtistId],
    ) -> Result<ReleaseId> {
        // ReleaseId
        // Para identificar un release, podemos usar su título y su primer artista principal.
        // Es una simplificación, pero efectiva para la mayoría de los casos.
        if let Some(first_artist_id) = album_artist_ids.first() {
            if let Some(id) = tx
                .query_row(
                    "SELECT r.id FROM releases r
                     JOIN release_main_artists rma ON r.id = rma.release_id
                     WHERE r.title = ?1 AND rma.artist_id = ?2",
                    params![title, first_artist_id],
                    |row| row.get(0),
                )
                .optional()?
            {
                return Ok(id);
            }
        }

        // Si no se encontró, se crea un nuevo release
        tx.execute(
            "INSERT INTO releases (title, format) VALUES (?1, ?2)",
            params![title, "Album"],
        )?;
        let release_id = tx.last_insert_rowid() as ReleaseId;

        // Vinculamos todos los artistas principales del álbum
        let mut stmt_artists =
            tx.prepare("INSERT INTO release_main_artists (release_id, artist_id) VALUES (?1, ?2)")?;
        for artist_id in album_artist_ids {
            stmt_artists.execute(params![release_id, artist_id])?;
        }

        Ok(release_id)
    }

    /// Busca una canción por título e intérprete; si no existe, la crea y añade sus créditos.
    fn find_or_create_song(
        tx: &rusqlite::Transaction,
        title: &str,
        performer_ids: &[ArtistId],
        featured_ids: &[ArtistId],
        composer_ids: &[ArtistId],
        producer_ids: &[ArtistId],
    ) -> Result<SongId> {
        // SongId
        if let Some(first_performer_id) = performer_ids.first() {
            if let Some(id) = tx
                .query_row(
                    "SELECT s.id FROM songs s
                     JOIN song_credits sc ON s.id = sc.song_id
                     WHERE s.title = ?1 AND sc.artist_id = ?2 AND sc.role = 'performer'",
                    params![title, first_performer_id],
                    |row| row.get(0),
                )
                .optional()?
            {
                return Ok(id);
            }
        }

        tx.execute("INSERT INTO songs (title) VALUES (?1)", [title])?;
        let song_id = tx.last_insert_rowid() as SongId;

        let mut stmt =
            tx.prepare("INSERT INTO song_credits (song_id, artist_id, role) VALUES (?1, ?2, ?3)")?;
        for id in performer_ids {
            stmt.execute(params![song_id, id, "performer"])?;
        }
        for id in featured_ids {
            stmt.execute(params![song_id, id, "featured"])?;
        }
        for id in composer_ids {
            stmt.execute(params![song_id, id, "composer"])?;
        }
        for id in producer_ids {
            stmt.execute(params![song_id, id, "producer"])?;
        }
        Ok(song_id)
    }

    /// Inserta una nueva fila en la tabla `release_tracks`.
    fn create_release_track(
        tx: &rusqlite::Transaction,
        song_id: SongId,
        release_id: ReleaseId,
        track_data: &UnresolvedTrack,
    ) -> Result<ReleaseTrackId> {
        tx.execute(
            "INSERT INTO release_tracks (
                    song_id, release_id, track_number, disc_number, path,
                    size_bytes, modified_timestamp, duration_seconds, bitrate_kbps,
                    sample_rate_hz, channels
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                song_id,
                release_id,
                track_data.track_number.unwrap_or(0),
                track_data.disc_number.unwrap_or(0),
                track_data.path.to_str(),
                track_data.file_size,
                track_data.last_modified,
                track_data.duration.as_secs_f64(),
                track_data.bitrate_kbps,
                track_data.sample_rate,
                track_data.channels,
            ],
        )?;

        Ok(tx.last_insert_rowid() as ReleaseTrackId)
    }

    /// Obtiene una lista de IDs y rutas de la cola de fingerprinting.
    pub fn get_fingerprint_queue(&self, limit: u32) -> Result<Vec<(ReleaseTrackId, PathBuf)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT fq.release_track_id, rt.path FROM fingerprint_queue fq
             JOIN release_tracks rt ON fq.release_track_id = rt.id
             LIMIT ?1",
        )?;

        let tracks = stmt
            .query_map([limit], |row| {
                let path_str: String = row.get(1)?;
                let path_buf = PathBuf::from(path_str);

                Ok((row.get(0)?, path_buf))
            })?
            .collect::<Result<Vec<(ReleaseTrackId, PathBuf)>, _>>()?;

        Ok(tracks)
    }

    /// Actualiza un ReleaseTrack con su fingerprint y lo saca de la cola.
    pub fn set_fingerprint_for_track(&self, track_id: ReleaseTrackId, fingerprint: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute(
            "UPDATE release_tracks SET fingerprint = ?1 WHERE id = ?2",
            params![fingerprint, track_id],
        )?;
        tx.execute(
            "DELETE FROM fingerprint_queue WHERE release_track_id = ?1",
            params![track_id],
        )?;

        tx.commit()?;
        Ok(())
    }
}

impl LocalStorage {
    /// Actualiza una canción con su AcoustID verificado y fusiona si es un duplicado.
    pub fn update_track_with_acoustid(
        &self,
        track_id_to_update: ReleaseTrackId,
        best_result: &AcoustidResult,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let acoustid = &best_result.id;

        // 1. ¿Ya existe una canción "maestra" con este AcoustID?
        let master_song_id: Option<SongId> = tx
            .query_row("SELECT id FROM songs WHERE acoustid = ?1", [acoustid], |row| {
                row.get(0)
            })
            .optional()?;

        // 2. Obtén el song_id del track que estamos procesando
        let current_song_id: SongId = tx.query_row(
            "SELECT song_id FROM release_tracks WHERE id = ?1",
            [track_id_to_update],
            |row| row.get(0),
        )?;

        if let Some(master_id) = master_song_id {
            if master_id != current_song_id {
                // ¡Duplicado encontrado! Fusionamos.
                info!(
                    "Duplicado encontrado. Fusionando song_id {} con {}",
                    current_song_id, master_id
                );
                // Reasignamos este track a la canción maestra.
                tx.execute(
                    "UPDATE release_tracks SET song_id = ?1 WHERE id = ?2",
                    params![master_id, track_id_to_update],
                )?;
                // Borramos la canción duplicada si ya no tiene más tracks.
                // (Una lógica más avanzada podría comprobar esto antes de borrar)
                tx.execute("DELETE FROM songs WHERE id = ?1", [current_song_id])?;
            }
        } else {
            // No es un duplicado, simplemente actualizamos la canción actual con su AcoustID.
            tx.execute(
                "UPDATE songs SET acoustid = ?1 WHERE id = ?2",
                params![acoustid, current_song_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_verification_queue(
        &self,
        limit: u32,
    ) -> Result<Vec<(ReleaseTrackId, PathBuf, std::time::Duration)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT rt.id, rt.path, rt.duration_seconds FROM release_tracks rt
             JOIN songs s ON rt.song_id = s.id
             WHERE rt.fingerprint IS NOT NULL AND s.acoustid IS NULL
             LIMIT ?1",
        )?;

        let tracks = stmt
            .query_map([limit], |row| {
                let path_str: String = row.get(1)?;
                let path_buf = PathBuf::from(path_str);
                let duration_secs: f64 = row.get(2)?;

                Ok((
                    row.get(0)?,
                    path_buf,
                    std::time::Duration::from_secs_f64(duration_secs),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }
}
