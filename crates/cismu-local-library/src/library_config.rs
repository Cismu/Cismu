use std::collections::HashMap;
use std::path::PathBuf;

use cismu_paths::{PATHS, UserDirs};
use config::{Config, File, FileFormat};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
use crate::extensions::{ExtensionConfig, SupportedExtension, default_extension_config};

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Eq)]
#[builder(setter(into, strip_option), default)]
#[serde(default)]
pub struct LibraryConfig {
    pub database: DatabaseConfig,
    pub scan: ScanConfig,
    pub extensions: HashMap<SupportedExtension, ExtensionConfig>,
    pub cover_art_dir: PathBuf,
    pub fingerprint: FingerprintAlgorithm,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        LibraryConfig {
            database: DatabaseConfig::default(),
            scan: ScanConfig::default(),
            extensions: default_extension_config(),
            cover_art_dir: PATHS.covers_dir.clone(),
            fingerprint: FingerprintAlgorithm::Chromaprint,
        }
    }
}

impl LibraryConfig {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_string_lossy().into_owned();
        let cfg = Config::builder()
            .add_source(File::new(&path, FileFormat::Toml))
            .build()
            .map_err(ConfigError::Parse)?;
        let lc = cfg
            .try_deserialize::<LibraryConfig>()
            .map_err(ConfigError::Parse)?;

        lc.validate()?;

        Ok(lc)
    }

    /// Comprueba que los campos cumplen tus invariantes.
    fn validate(&self) -> Result<(), ConfigError> {
        // threads > 0
        if self.scan.threads == 0 {
            return Err(ConfigError::Validation("scan.threads must be > 0".into()));
        }

        // directorio de portadas existe y es directorio
        if !self.cover_art_dir.exists() {
            return Err(ConfigError::Validation(format!(
                "cover_art_dir '{}' does not exist",
                self.cover_art_dir.display()
            )));
        } else if !self.cover_art_dir.is_dir() {
            return Err(ConfigError::Validation(format!(
                "cover_art_dir '{}' is not a directory",
                self.cover_art_dir.display()
            )));
        }

        // Validamos cada extensión
        for (ext, cfg) in &self.extensions {
            cfg.validate().map_err(|e| {
                ConfigError::Validation(format!("extension {} invalid: {}", ext, e))
            })?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "path")]
pub enum DatabaseConfig {
    #[serde(rename = "sqlite")]
    Sqlite(PathBuf),
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig::Sqlite(PATHS.library_db.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ScanConfig {
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
    pub follow_symlinks: bool,
    pub threads: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let include_dir = UserDirs::new()
            .and_then(|ud| ud.audio_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        ScanConfig {
            include: vec![include_dir],
            exclude: Vec::new(),
            follow_symlinks: true,
            threads: num_cpus::get(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FingerprintAlgorithm {
    Chromaprint,
}

impl Default for FingerprintAlgorithm {
    fn default() -> Self {
        FingerprintAlgorithm::Chromaprint
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytesize::ByteSize;
    use std::time::Duration;
    use std::{fs::File, io::Write, path::PathBuf};
    use tempfile::tempdir;

    /// Helper: escribe un string TOML en un archivo y devuelve la ruta.
    fn write_toml(toml: &str, dir: &PathBuf) -> PathBuf {
        let path = dir.join("test.toml");
        let mut f = File::create(&path).expect("no pudo crear test.toml");
        write!(f, "{}", toml).expect("no pudo escribir en test.toml");
        path
    }

    #[test]
    fn default_library_config_has_valid_paths_and_values() {
        let def = LibraryConfig::default();

        // Base de datos apunta a PATHS.library_db
        assert_eq!(
            def.database,
            DatabaseConfig::Sqlite(PATHS.library_db.clone())
        );

        // cover_art_dir igual a PATHS.covers_dir
        assert_eq!(def.cover_art_dir, PATHS.covers_dir.clone());

        // scan.include contiene al menos un PathBuf
        assert!(!def.scan.include.is_empty());

        // threads > 0
        assert!(def.scan.threads >= 1);

        // fingerprint Chromaprint por defecto
        assert_eq!(def.fingerprint, FingerprintAlgorithm::Chromaprint);

        // extensions no está vacío (tiene la configuración por defecto)
        assert!(!def.extensions.is_empty());
    }

    #[test]
    fn from_file_empty_toml_uses_all_defaults() {
        let tmp = tempdir().unwrap();
        let empty = write_toml("", &tmp.path().to_path_buf());

        // Cargar desde un archivo sin contenidos TOML
        let cfg = LibraryConfig::from_file(&empty).expect("no cargó config desde TOML vacío");
        let def = LibraryConfig::default();

        assert_eq!(cfg, def);
    }

    #[test]
    fn from_file_partial_toml_overrides_only_specified_fields() {
        let tmp = tempdir().unwrap();

        // 1) Creamos un subdirectorio válido para cover_art_dir
        let cover_dir = tmp.path().join("mi_cover");
        std::fs::create_dir_all(&cover_dir).unwrap();

        // 2) Escribimos el TOML referenciando esa ruta dinámica
        let toml = format!(
            r#"
cover_art_dir = "{}"

[scan]
threads = 8
"#,
            cover_dir.display()
        );
        let path = write_toml(&toml, &tmp.path().to_path_buf());

        // 3) Hacemos el parse + validate
        let cfg = LibraryConfig::from_file(&path).unwrap();
        let def = LibraryConfig::default();

        // cover_art_dir viene del TOML y existe
        assert_eq!(cfg.cover_art_dir, cover_dir);

        // scan.threads viene del TOML
        assert_eq!(cfg.scan.threads, 8);

        // el resto sigue en los defaults
        assert_eq!(cfg.scan.include, def.scan.include);
        assert_eq!(cfg.scan.exclude, def.scan.exclude);
        assert_eq!(cfg.database, def.database);
        assert_eq!(cfg.fingerprint, def.fingerprint);
        assert_eq!(cfg.extensions, def.extensions);
    }

    #[test]
    fn from_file_full_toml_all_fields_override() {
        let tmp = tempdir().unwrap();
        let toml = format!(
            r#"
            #[database]
            [database]
            type = "sqlite"
            path = "{}"

            [scan]
            include = ["/uno", "/dos"]
            exclude = ["/tmp"]
            follow_symlinks = false
            threads = 4

            [extensions]
            # aquí iría tu serialización de default_extension_config()
            # por simplicidad, lo dejamos vacío
            "#,
            tmp.path().join("otra.db").to_string_lossy()
        );
        let path = write_toml(&toml, &tmp.path().to_path_buf());
        let cfg = LibraryConfig::from_file(&path).unwrap();

        // database con nueva ruta
        assert_eq!(
            cfg.database,
            DatabaseConfig::Sqlite(tmp.path().join("otra.db"))
        );

        // scan.override
        assert_eq!(
            cfg.scan.include,
            vec![PathBuf::from("/uno"), PathBuf::from("/dos")]
        );
        assert_eq!(cfg.scan.exclude, vec![PathBuf::from("/tmp")]);
        assert_eq!(cfg.scan.follow_symlinks, false);
        assert_eq!(cfg.scan.threads, 4);
    }

    #[test]
    fn extension_config_validate_rejects_zero_file_size() {
        let cfg = ExtensionConfig {
            min_file_size: ByteSize::b(0),
            min_duration: Duration::from_secs(30),
        };
        let err = cfg.validate().unwrap_err();
        assert_eq!(err, "min_file_size must be greater than zero");
    }

    #[test]
    fn extension_config_validate_rejects_zero_duration() {
        let cfg = ExtensionConfig {
            min_file_size: ByteSize::kib(500),
            min_duration: Duration::from_secs(0),
        };
        let err = cfg.validate().unwrap_err();
        assert_eq!(err, "min_duration must be greater than zero");
    }

    #[test]
    fn library_config_validate_rejects_zero_threads() {
        let mut cfg = LibraryConfig::default();
        cfg.scan.threads = 0;
        let err = cfg.validate().unwrap_err();
        assert!(
            matches!(&err, ConfigError::Validation(msg) if msg.contains("scan.threads must be > 0")),
            "esperaba error de threads > 0, fue: {:?}",
            err
        );
    }

    #[test]
    fn library_config_validate_rejects_missing_cover_art_dir() {
        let mut cfg = LibraryConfig::default();
        cfg.cover_art_dir = PathBuf::from("/ruta/que/no/existe");
        let err = cfg.validate().unwrap_err();
        assert!(
            matches!(&err, ConfigError::Validation(msg) if msg.contains("does not exist")),
            "esperaba error de cover_art_dir inexistente, fue: {:?}",
            err
        );
    }

    #[test]
    fn library_config_validate_rejects_cover_art_not_directory() {
        // Creamos un archivo regular en un tempdir
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("algo.txt");
        File::create(&file_path).unwrap();
        let mut cfg = LibraryConfig::default();
        cfg.cover_art_dir = file_path.clone();
        let err = cfg.validate().unwrap_err();
        assert!(
            matches!(&err, ConfigError::Validation(msg) if msg.contains("is not a directory")),
            "esperaba error de cover_art_dir no-dir, fue: {:?}",
            err
        );
    }

    #[test]
    fn library_config_validate_rejects_invalid_extension_cfg() {
        let mut cfg = LibraryConfig::default();
        // Ponemos un ExtensionConfig inválido (tamaño = 0)
        cfg.extensions.insert(
            SupportedExtension::Mp3,
            ExtensionConfig {
                min_file_size: ByteSize::b(0),
                min_duration: Duration::from_secs(30),
            },
        );
        let err = cfg.validate().unwrap_err();
        assert!(
            matches!(&err, ConfigError::Validation(msg) if msg.contains("extension mp3 invalid")),
            "esperaba error de extensión inválida, fue: {:?}",
            err
        );
    }
}
