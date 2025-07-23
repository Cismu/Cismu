use std::{env, fs::File, path::PathBuf};

use crate::{errors::Error, fs_utils};
use tracing::{Level, error, info, instrument};

use directories::ProjectDirs;

/// Name of the ENV var for base route override (“portable” mode)
const ENV_BASE_DIR: &str = "CISMU_BASE_DIR";

/// Container of all important paths and files of the app
#[derive(Debug)]
pub struct CismuPaths {
    // Directories
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub waveforms_dir: PathBuf,
    pub lyrics_dir: PathBuf,

    // Files
    pub settings_file: PathBuf,
    pub ui_file: PathBuf,
    pub keybindings_file: PathBuf,
    pub log_file: PathBuf,
    pub lock_file: PathBuf,

    // Databases
    pub library_db: PathBuf,
    pub playlists_db: PathBuf,
}

impl CismuPaths {
    #[instrument(name = "CismuPaths::new", level = Level::INFO, err, skip_all, fields(mode = %env::var(ENV_BASE_DIR).map(|_| "portable").unwrap_or("default")))]
    pub fn new() -> Result<Self, Error> {
        let (config_dir, data_dir, cache_dir) = if let Ok(base) = env::var(ENV_BASE_DIR) {
            info!("Using portable mode");
            let b = PathBuf::from(base);
            (b.join("config"), b.join("data"), b.join("cache"))
        } else {
            let proj = ProjectDirs::from("org", "Cismu", "Cismu").ok_or(Error::NoHome)?;
            info!("Using default mode");
            (
                proj.config_dir().to_path_buf(),
                proj.data_dir().to_path_buf(),
                proj.cache_dir().to_path_buf(),
            )
        };

        let paths = CismuPaths {
            config_dir: config_dir.clone(),
            settings_file: config_dir.join("settings.toml"),
            ui_file: config_dir.join("ui.toml"),
            keybindings_file: config_dir.join("keybindings.toml"),

            data_dir: data_dir.clone(),
            library_db: data_dir.join("library.db"),
            playlists_db: data_dir.join("playlists.db"),
            state_dir: data_dir.join("state"),
            logs_dir: data_dir.join("logs"),
            log_file: data_dir.join("cismu.log"),

            cache_dir: cache_dir.clone(),
            covers_dir: cache_dir.join("covers"),
            waveforms_dir: cache_dir.join("waveforms"),
            lyrics_dir: cache_dir.join("lyrics"),

            lock_file: data_dir.join("cismu.lock"),
        };

        paths.ensure_structure()?;
        paths.validate_structure()?;

        Ok(paths)
    }

    /// Devuelve true si este es el primer arranque (cismu.lock no existía)
    #[instrument(level = Level::DEBUG, skip(self), ret)]
    pub fn is_first_run(&self) -> bool {
        !self.lock_file.exists()
    }

    /// Crea (si no existe) y adquiere un advisory-lock excluyente en cismu.lock.
    /// Mantén vivo el File retornado para conservar el lock.
    #[instrument(level = Level::TRACE, err, skip(self), fields(lock_file = %self.lock_file.display()))]
    pub fn lock(&self) -> Result<File, Error> {
        fs_utils::lock_file(&self.lock_file)
    }
}

impl CismuPaths {
    /// Devuelve la ruta final de un cover dado su hash hex y la extensión.
    ///
    /// Estructura:
    ///   <cache_dir>/covers/<1º nibble>/<2 primeros nibbles>/<hash>.<ext>
    #[instrument(level = Level::TRACE, err, skip(self, hash, ext), fields(hash, ext))]
    pub fn cover_path(&self, base_dir: PathBuf, hash: &str, ext: &str) -> Result<PathBuf, Error> {
        let hex = hash.to_lowercase();

        // 1) Validaciones
        if hex.len() < 2 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            error!("Invalid cover hash: {}", hash);
            return Err(Error::InvalidCoverHash(hash.to_string()));
        }

        // 2) Calculamos subcarpetas
        let d1 = &hex[0..1];
        let d2 = &hex[0..2];

        // 3) Formamos el PathBuf
        Ok(base_dir.join(d1).join(d2).join(format!("{}.{}", hex, ext.trim_start_matches('.'))))
    }

    /// Asegura que la carpeta del cover existe y devuelve la ruta completa lista para escribir.
    #[instrument(level = Level::TRACE, err, skip(self, hash, ext), fields(hash, ext))]
    pub fn ensure_cover_path(&self, base_dir: PathBuf, hash: &str, ext: &str) -> Result<PathBuf, Error> {
        let path = self.cover_path(base_dir, hash, ext)?;
        if let Some(parent) = path.parent() {
            fs_utils::ensure_dir(parent)?;
        }
        Ok(path)
    }
}

impl CismuPaths {
    /// Se asegura de que TODOS los dirs y ficheros básicos existen.
    #[instrument(level = Level::INFO, err, skip(self))]
    pub fn ensure_structure(&self) -> Result<(), Error> {
        info!("Ensuring structure");

        // carpetas
        fs_utils::ensure_dir(&self.config_dir)?;
        fs_utils::ensure_dir(&self.data_dir)?;
        fs_utils::ensure_dir(&self.cache_dir)?;
        fs_utils::ensure_dir(&self.state_dir)?;
        fs_utils::ensure_dir(&self.logs_dir)?;
        fs_utils::ensure_dir(&self.covers_dir)?;
        fs_utils::ensure_dir(&self.waveforms_dir)?;
        fs_utils::ensure_dir(&self.lyrics_dir)?;

        // ficheros
        fs_utils::ensure_file(&self.settings_file)?;
        fs_utils::ensure_file(&self.ui_file)?;
        fs_utils::ensure_file(&self.keybindings_file)?;
        fs_utils::ensure_file(&self.playlists_db)?;
        fs_utils::ensure_file(&self.log_file)?;

        Ok(())
    }

    /// Valida que cada ruta existe Y es escribible. Si falta, la intenta crear.
    /// Si no tiene permisos de escritura, retorna Err.
    #[instrument(level = Level::INFO, err, skip(self))]
    pub fn validate_structure(&self) -> Result<(), Error> {
        info!("Validating structure");

        let all_paths = vec![
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.state_dir,
            &self.logs_dir,
            &self.covers_dir,
            &self.waveforms_dir,
            &self.lyrics_dir,
        ];

        for dir in all_paths {
            if !dir.exists() {
                fs_utils::ensure_dir(dir)?;
            }
            // chequea permisos de escritura:
            fs_utils::check_writable(dir)?;
        }

        Ok(())
    }
}
