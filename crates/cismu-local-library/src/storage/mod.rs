mod embedded;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cismu_core::discography::{
    artist::{Artist, ArtistId},
    release::{Release, ReleaseId, ReleaseType},
    song::SongId,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use tracing::{info, trace};

use cismu_paths::PATHS;

use embedded::migrations::runner;

use crate::parsing::UnresolvedTrack;

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
    /// Orquesta el proceso completo para resolver una pista no resuelta.
    pub fn resolve_unresolved_track(&self, track: UnresolvedTrack) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // 1. Resolver todos los artistas. Pura lógica de negocio + llamada a queries.
        let artist_map = self.resolve_all_artists(&tx, &track)?;

        // 2. Resolver el lanzamiento. Pura lógica de negocio + llamada a queries.
        let release_id = self.resolve_release(&tx, &track, &artist_map)?;

        // 3. Resolver la canción abstracta
        let song_id = self.resolve_song(&tx, &track, &artist_map)?;

        // 4. Insertar la pista física que une todo (PASO FINAL)
        queries::insert_release_track(&tx, &track, song_id, release_id)?;

        tx.commit()?;
        Ok(())
    }

    /// Prepara la lista de nombres de artistas y llama al query correspondiente.
    fn resolve_all_artists(
        &self,
        tx: &Transaction,
        track: &UnresolvedTrack,
    ) -> Result<HashMap<String, ArtistId>> {
        let mut all_names: Vec<String> = track
            .release_artists
            .iter()
            .chain(track.track_performers.iter())
            .chain(track.track_featured.iter())
            .chain(track.track_composers.iter())
            .chain(track.track_producers.iter())
            .cloned()
            .collect();

        all_names.sort();
        all_names.dedup();
        all_names.retain(|n| !n.is_empty()); // Asegurarse de no procesar nombres vacíos

        let ids = queries::find_or_create_artists(tx, &all_names)?;
        let map = all_names.into_iter().zip(ids.into_iter()).collect();
        Ok(map)
    }

    /// Prepara los datos del lanzamiento y coordina la búsqueda o creación en la base de datos.
    fn resolve_release(
        &self,
        tx: &Transaction,
        track: &UnresolvedTrack,
        artist_map: &HashMap<String, ArtistId>,
    ) -> Result<ReleaseId> {
        let release_title = track.release_title.as_deref().unwrap_or("Unknown Release");

        let target_artist_ids: Vec<ArtistId> = track
            .release_artists
            .iter()
            .filter_map(|name| artist_map.get(name).copied())
            .collect();

        // Llama a la función de búsqueda en la capa de queries.
        if let Some(id) = queries::find_release_by_artists(tx, release_title, &target_artist_ids)? {
            return Ok(id);
        }

        // Si no se encontró, preparamos los datos para la creación.
        let release_types = track
            .release_type
            .as_deref()
            .map(ReleaseType::parse)
            .unwrap_or_default();
        let format_string = release_types
            .iter()
            .map(|rt| format!("{:?}", rt))
            .collect::<Vec<_>>()
            .join(";");
        let final_format_string = if format_string.is_empty() {
            "Other".to_string()
        } else {
            format_string
        };

        // Llama a la función de creación en la capa de queries.
        let release_id = queries::create_new_release(
            tx,
            release_title,
            &final_format_string,
            track.release_date.as_deref(),
            &target_artist_ids,
        )?;
        Ok(release_id)
    }

    /// Prepara los datos de la canción y coordina la búsqueda o creación en la base de datos.
    fn resolve_song(
        &self,
        tx: &Transaction,
        track: &UnresolvedTrack,
        artist_map: &HashMap<String, ArtistId>,
    ) -> Result<SongId> {
        let track_title = match track.track_title.as_deref() {
            Some(title) if !title.is_empty() => title,
            _ => return Err(anyhow::anyhow!("La pista no tiene título")), // Una canción abstracta debe tener título
        };

        let performer_ids: Vec<ArtistId> = track
            .track_performers
            .iter()
            .filter_map(|name| artist_map.get(name).copied())
            .collect();

        // Llama a la función de búsqueda en la capa de queries.
        if let Some(id) = queries::find_song_by_performers(tx, track_title, &performer_ids)? {
            return Ok(id);
        }

        // Si no se encontró, preparamos todos los créditos para la creación.
        let featured_ids: Vec<ArtistId> = track
            .track_featured
            .iter()
            .filter_map(|n| artist_map.get(n).copied())
            .collect();
        let composer_ids: Vec<ArtistId> = track
            .track_composers
            .iter()
            .filter_map(|n| artist_map.get(n).copied())
            .collect();
        let producer_ids: Vec<ArtistId> = track
            .track_producers
            .iter()
            .filter_map(|n| artist_map.get(n).copied())
            .collect();

        // Llama a la función de creación en la capa de queries.
        let song_id = queries::create_new_song(
            tx,
            track_title,
            &performer_ids,
            &featured_ids,
            &composer_ids,
            &producer_ids,
        )?;
        Ok(song_id)
    }
}

impl LocalStorage {
    /// Devuelve una lista de todos los artistas en la biblioteca.
    pub fn get_all_artists(&self) -> Result<Vec<Artist>> {
        let conn = self.conn.lock().unwrap();
        queries::get_all_artists(&conn)
    }

    /// Devuelve una lista de todos los lanzamientos de un artista específico.
    pub fn get_releases_for_artist(&self, artist_id: ArtistId) -> Result<Vec<Release>> {
        let conn = self.conn.lock().unwrap();
        queries::get_releases_for_artist(&conn, artist_id)
    }

    /// Devuelve los detalles completos de un lanzamiento, incluyendo su lista de pistas.
    pub fn get_release_details(&self, release_id: ReleaseId) -> Result<Option<Release>> {
        let conn = self.conn.lock().unwrap();
        queries::get_release_details(&conn, release_id)
    }
}

mod queries {
    use cismu_core::discography::release_track::ReleaseTrackId;

    use super::*;

    pub fn find_or_create_artists(tx: &Transaction, artist_names: &[String]) -> rusqlite::Result<Vec<ArtistId>> {
        let mut stmt_select = tx.prepare(
            "SELECT id
               FROM artists
              WHERE TRIM(name) = TRIM(?1) COLLATE NOCASE",
        )?;

        let mut stmt_insert = tx.prepare("INSERT INTO artists (name) VALUES (?1)")?;
        let mut artist_ids = Vec::with_capacity(artist_names.len());

        for name in artist_names {
            if let Some(id) = stmt_select.query_row([name], |row| row.get::<usize, ArtistId>(0)).optional()? {
                artist_ids.push(id);
            } else {
                stmt_insert.execute([name])?;
                artist_ids.push(tx.last_insert_rowid() as ArtistId);
            }
        }

        Ok(artist_ids)
    }

    pub fn find_release_by_artists(
        tx: &Transaction,
        title: &str,
        target_artists: &[ArtistId],
    ) -> Result<Option<ReleaseId>> {
        if target_artists.is_empty() {
            return Ok(None);
        }

        let mut stmt_find_releases = tx.prepare("SELECT id FROM releases WHERE title = ?1")?;
        let candidate_ids: Vec<ReleaseId> = stmt_find_releases
            .query_map([title], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        let mut stmt_get_artists =
            tx.prepare("SELECT artist_id FROM release_main_artists WHERE release_id = ?1")?;

        for release_id in candidate_ids {
            let mut db_artists: Vec<ArtistId> = stmt_get_artists
                .query_map([release_id], |row| row.get(0))?
                .collect::<Result<_, _>>()?;

            let mut target_artists_sorted = target_artists.to_vec();
            target_artists_sorted.sort_unstable();
            db_artists.sort_unstable();

            if target_artists_sorted == db_artists {
                return Ok(Some(release_id));
            }
        }

        Ok(None)
    }

    pub fn create_new_release(
        tx: &Transaction,
        title: &str,
        format: &str,
        date: Option<&str>,
        artists: &[ArtistId],
    ) -> Result<ReleaseId> {
        tx.execute(
            "INSERT INTO releases (title, format, release_date) VALUES (?1, ?2, ?3)",
            params![title, format, date],
        )?;
        let release_id = tx.last_insert_rowid() as ReleaseId;

        let mut stmt_link_artist =
            tx.prepare("INSERT INTO release_main_artists (release_id, artist_id) VALUES (?1, ?2)")?;
        for artist_id in artists {
            stmt_link_artist.execute(params![release_id, artist_id])?;
        }

        Ok(release_id)
    }

    pub fn find_song_by_performers(
        tx: &Transaction,
        title: &str,
        target_performers: &[ArtistId],
    ) -> Result<Option<SongId>> {
        if target_performers.is_empty() {
            return Ok(None);
        }
        // Lógica para encontrar una canción por título y conjunto exacto de intérpretes...
        // Es muy similar a `find_release_by_artists`, pero buscando en `songs` y `song_credits`.
        let mut stmt_find_songs = tx.prepare("SELECT id FROM songs WHERE title = ?1")?;
        let candidates: Vec<SongId> = stmt_find_songs
            .query_map([title], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        let mut stmt_get_performers =
            tx.prepare("SELECT artist_id FROM song_credits WHERE song_id = ?1 AND role = 'performer'")?;
        for song_id in candidates {
            let mut db_performers: Vec<ArtistId> = stmt_get_performers
                .query_map([song_id], |row| row.get(0))?
                .collect::<Result<_, _>>()?;

            let mut target_performers_sorted = target_performers.to_vec();
            target_performers_sorted.sort_unstable();
            db_performers.sort_unstable();

            if target_performers_sorted == db_performers {
                return Ok(Some(song_id));
            }
        }

        Ok(None)
    }

    pub fn create_new_song(
        tx: &Transaction,
        title: &str,
        performers: &[ArtistId],
        featured: &[ArtistId],
        composers: &[ArtistId],
        producers: &[ArtistId],
    ) -> Result<SongId> {
        tx.execute("INSERT INTO songs (title) VALUES (?1)", [title])?;
        let song_id = tx.last_insert_rowid() as SongId;

        let mut stmt =
            tx.prepare("INSERT INTO song_credits (song_id, artist_id, role) VALUES (?1, ?2, ?3)")?;

        for artist_id in performers {
            stmt.execute(params![song_id, artist_id, "performer"])?;
        }
        for artist_id in featured {
            stmt.execute(params![song_id, artist_id, "featured"])?;
        }
        for artist_id in composers {
            stmt.execute(params![song_id, artist_id, "composer"])?;
        }
        for artist_id in producers {
            stmt.execute(params![song_id, artist_id, "producer"])?;
        }

        Ok(song_id)
    }

    pub fn insert_release_track(
        tx: &Transaction,
        track: &UnresolvedTrack,
        song_id: SongId,
        release_id: ReleaseId,
    ) -> Result<()> {
        tx.execute(
            "INSERT OR REPLACE INTO release_tracks (song_id, release_id, track_number, disc_number, path, size_bytes, modified_timestamp, duration_seconds, bitrate_kbps, sample_rate_hz, channels)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                song_id,
                release_id,
                track.track_number,
                track.disc_number,
                track.path.to_str(),
                track.file_size,
                track.last_modified,
                track.duration.as_secs_f64(),
                track.bitrate_kbps,
                track.sample_rate,
                track.channels,
            ]
        )?;
        Ok(())
    }

    /// Consulta la base de datos para obtener todos los artistas.
    pub fn get_all_artists(conn: &Connection) -> Result<Vec<Artist>> {
        let mut stmt = conn.prepare("SELECT id, name FROM artists ORDER BY name COLLATE NOCASE")?;

        let artists_iter = stmt.query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                ..Default::default()
            })
        })?;

        let mut artists = Vec::new();
        for artist in artists_iter {
            artists.push(artist?);
        }
        Ok(artists)
    }

    /// Consulta los lanzamientos asociados a un `ArtistId`.
    pub fn get_releases_for_artist(conn: &Connection, artist_id: ArtistId) -> Result<Vec<Release>> {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.title, r.release_date FROM releases r
             JOIN release_main_artists rma ON r.id = rma.release_id
             WHERE rma.artist_id = ?1
             ORDER BY r.release_date DESC",
        )?;

        let releases_iter = stmt.query_map([artist_id], |row| {
            Ok(Release {
                id: row.get(0)?,
                title: row.get(1)?,
                release_date: row.get(2)?,
                // En esta vista simplificada, no cargamos toda la lista de pistas o artistas.
                ..Default::default()
            })
        })?;

        let mut releases = Vec::new();
        for release in releases_iter {
            releases.push(release?);
        }
        Ok(releases)
    }

    /// Carga un `Release` completo desde la base de datos, incluyendo artistas y pistas.
    pub fn get_release_details(conn: &Connection, release_id: ReleaseId) -> Result<Option<Release>> {
        // 1. Obtener los datos base del release
        let mut release: Release = match conn
            .query_row(
                "SELECT id, title, format, release_date FROM releases WHERE id = ?1",
                [release_id],
                |row| {
                    Ok(Release {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        // Usamos la función de parseo que ya tienes en tu enum ReleaseType
                        release_type: ReleaseType::parse(&row.get::<_, String>(2)?),
                        release_date: row.get(3)?,
                        ..Default::default()
                    })
                },
            )
            .optional()?
        {
            Some(r) => r,
            None => return Ok(None), // Si no se encuentra el release, devuelve None
        };

        // 2. Cargar los artistas principales del release
        let mut stmt_artists =
            conn.prepare("SELECT artist_id FROM release_main_artists WHERE release_id = ?1")?;
        let artist_ids = stmt_artists
            .query_map([release_id], |row| row.get(0))?
            .collect::<Result<Vec<ArtistId>, _>>()?;
        release.main_artist_ids = artist_ids;

        // 3. Cargar las pistas del release (uniendo con songs para obtener el título)
        let mut stmt_tracks = conn.prepare(
            "SELECT rt.id, rt.song_id, s.title, rt.track_number, rt.disc_number, rt.duration_seconds, rt.path
             FROM release_tracks rt
             JOIN songs s ON rt.song_id = s.id
             WHERE rt.release_id = ?1
             ORDER BY rt.disc_number, rt.track_number",
        )?;

        // Nota: Aquí estamos creando una estructura simplificada de la pista para la UI.
        // Deberías definir un struct para esto, pero por ahora lo hacemos anónimo.
        let tracks_iter = stmt_tracks.query_map([release_id], |row| {
            // Aquí deberías construir tu struct `ReleaseTrack` completo.
            // Por simplicidad, solo mostramos cómo se obtienen los datos.
            let id: ReleaseTrackId = row.get(0)?;
            // ... cargar el resto de los datos en tu struct `ReleaseTrack` ...
            Ok(id) // Placeholder
        })?;

        release.release_tracks = tracks_iter.collect::<Result<Vec<ReleaseTrackId>, _>>()?;

        // 4. (Opcional) Cargar artworks, géneros, etc. de la misma forma.

        Ok(Some(release))
    }
}
