//! Crate `cismu_paths`: rutas, locks y preferencias de Cismu

mod errors;
mod fs_utils;
mod paths;

pub use errors::Error;
pub use paths::CismuPaths;

pub use directories::UserDirs;
use once_cell::sync::Lazy;

/// Singleton global, para usar en todo el crate sin repetir `new()`
pub static PATHS: Lazy<CismuPaths> =
    Lazy::new(|| CismuPaths::new().expect("Failed to initialize CismuPaths"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// RAII-guard que setea y luego restaura (o elimina) una variable de entorno.
    struct EnvVarGuard {
        key: String,
        original: Option<String>,
    }

    impl EnvVarGuard {
        /// Guarda el valor actual de `key` (si existe), y luego la setea a `value`.
        fn new(key: &str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            // set_var es unsafe en Unix:
            unsafe { std::env::set_var(key, value) };
            EnvVarGuard {
                key: key.to_owned(),
                original,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(val) => unsafe { std::env::set_var(&self.key, val) },
                None => unsafe { std::env::remove_var(&self.key) },
            }
        }
    }

    #[test]
    fn cover_path_ok() {
        let tmp = tempdir().unwrap();
        let _env = EnvVarGuard::new("CISMU_BASE_DIR", tmp.path().to_str().unwrap());

        let paths = CismuPaths::new().unwrap();
        let p = paths.cover_path("1a47929b", "jpg").unwrap();

        let expected: PathBuf = tmp
            .path()
            .join("cache")
            .join("covers")
            .join("1")
            .join("1a")
            .join("1a47929b.jpg");
        assert_eq!(p, expected);
    }

    #[test]
    fn cover_path_too_short() {
        let tmp = tempdir().unwrap();
        let _env = EnvVarGuard::new("CISMU_BASE_DIR", tmp.path().to_str().unwrap());

        let paths = CismuPaths::new().unwrap();
        let err = paths.cover_path("f", "png").unwrap_err();
        match err {
            Error::InvalidCoverHash(h) => assert_eq!(h, "f".to_string()),
            _ => panic!("Esperaba InvalidCoverHash"),
        }
    }

    #[test]
    fn cover_path_non_hex() {
        let tmp = tempdir().unwrap();
        let _env = EnvVarGuard::new("CISMU_BASE_DIR", tmp.path().to_str().unwrap());

        let paths = CismuPaths::new().unwrap();
        let err = paths.cover_path("zzzz", "png").unwrap_err();
        match err {
            Error::InvalidCoverHash(h) => assert_eq!(h, "zzzz".to_string()),
            _ => panic!("Esperaba InvalidCoverHash"),
        }
    }

    #[test]
    fn new_creates_structure_and_lock_behavior() {
        let tmp = tempdir().unwrap();
        let _env = EnvVarGuard::new("CISMU_BASE_DIR", tmp.path().to_str().unwrap());

        let paths = CismuPaths::new().unwrap();

        // Carpetas base creadas
        assert!(paths.config_dir.exists());
        assert!(paths.data_dir.exists());
        assert!(paths.cache_dir.exists());

        // Ficheros básicos existen, pero lock aún no
        assert!(paths.settings_file.exists());
        assert!(paths.ui_file.exists());
        assert!(paths.keybindings_file.exists());
        assert!(paths.library_db.exists());
        assert!(paths.playlists_db.exists());
        assert!(paths.log_file.exists());
        assert!(!paths.lock_file.exists());

        // is_first_run → true
        assert!(paths.is_first_run());

        // lock() crea y bloquea el lock_file
        let _lock_handle = paths.lock().unwrap();
        assert!(paths.lock_file.exists());
        // ahora is_first_run → false
        assert!(!paths.is_first_run());
    }

    #[test]
    fn validate_structure_recreates_missing_dirs() {
        let tmp = tempdir().unwrap();
        let _env = EnvVarGuard::new("CISMU_BASE_DIR", tmp.path().to_str().unwrap());

        let paths = CismuPaths::new().unwrap();

        // Simula borrado de cache_dir en caliente
        std::fs::remove_dir_all(&paths.cache_dir).unwrap();
        assert!(!paths.cache_dir.exists());

        // validate_structure vuelve a crearla
        paths.validate_structure().unwrap();
        assert!(paths.cache_dir.exists());
        // Y subcarpetas que dependen de ella
        assert!(paths.waveforms_dir.exists());
        assert!(paths.lyrics_dir.exists());
    }
}
