use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use jwalk::WalkDir;
use rayon::prelude::*;

use super::config::LibraryConfig;
use super::traits::Scanner;
use super::utils::AudioFormat;

const ENV_VARS_WITH_DEFAULTS: &[(&str, &str)] = &[
    ("WINDIR", "C:\\Windows"),
    ("ProgramFiles", "C:\\Program Files"),
    ("ProgramFiles(x86)", "C:\\Program Files (x86)"),
    ("ProgramData", "C:\\ProgramData"),
    ("LOCALAPPDATA", ""),
    ("APPDATA", ""),
    ("TEMP", ""),
];

const RELATIVE_EXCLUSIONS: &[&str] = &[
    "$Recycle.Bin",
    "System Volume Information",
    "Recovery",
    "PerfLogs",
];

#[derive(Debug, Default)]
pub struct DefaultScanner;

impl DefaultScanner {
    /// Obtiene la raíz (unidad o “/”) de una ruta dada.
    fn root_of_path(path: &Path) -> String {
        match path.components().next() {
            Some(Component::Prefix(p)) => p.as_os_str().to_string_lossy().into_owned(),
            Some(Component::RootDir) => "/".to_string(),
            _ => "/".to_string(),
        }
    }

    /// Construye los conjuntos de rutas a escanear y a excluir.
    fn process_library_paths(
        &self,
        config: &LibraryConfig,
    ) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
        // Rutas base a escanear
        let library_paths = config
            .scan_directories
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        // Comenzamos con las exclusiones del config
        let mut excluded_paths = config
            .excluded_directories
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        // Añadimos variables de entorno con valor por defecto
        for &(var, default) in ENV_VARS_WITH_DEFAULTS {
            let val = env::var(var).unwrap_or_else(|_| default.to_string());
            if !val.is_empty() {
                excluded_paths.insert(PathBuf::from(val));
            }
        }

        // Añadimos exclusiones relativas bajo cada raíz de disco
        let mut seen_roots = HashSet::new();
        for lib in &library_paths {
            if let Some(root) = lib.ancestors().last().map(PathBuf::from) {
                if seen_roots.insert(root.clone()) {
                    for rel in RELATIVE_EXCLUSIONS {
                        excluded_paths.insert(root.join(rel));
                    }
                }
            }
        }

        (library_paths, excluded_paths)
    }

    /// Comprueba si la entrada es un fichero de audio conocido.
    fn is_audio_file(entry: &jwalk::DirEntry<((), ())>) -> bool {
        entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .and_then(|ext| AudioFormat::from_extension(ext))
                .is_some()
    }

    /// Filtra antes de descender en subdirectorios, eliminando los paths excluidos.
    fn filter_read_dir(
        excluded: &HashSet<PathBuf>,
        children: &mut Vec<Result<jwalk::DirEntry<((), ())>, jwalk::Error>>,
    ) {
        children.retain(|res| {
            if let Ok(entry) = res {
                !excluded.iter().any(|ex| entry.path().starts_with(ex))
            } else {
                false
            }
        });
    }

    /// Escanea recursivamente un directorio base y acumula ficheros de audio.
    fn scan_base_dir(
        &self,
        base: &Path,
        excluded: Arc<HashSet<PathBuf>>,
        found: Arc<Mutex<HashSet<PathBuf>>>,
        follow_symlinks: bool,
    ) {
        let walker = WalkDir::new(base)
            .follow_links(follow_symlinks)
            .process_read_dir({
                let excluded = excluded.clone();
                move |_depth, _path, _state, children| {
                    Self::filter_read_dir(&*excluded, children);
                }
            })
            .into_iter();

        for result in walker {
            if let Ok(entry) = result {
                if Self::is_audio_file(&entry) {
                    let mut guard = found.lock().unwrap();
                    guard.insert(entry.path().to_path_buf());
                }
            }
        }
    }
}

impl Scanner for DefaultScanner {
    fn scan(&self, config: &LibraryConfig) -> HashMap<String, HashSet<PathBuf>> {
        // Prepara rutas a escanear y excluir
        let (library_paths, excluded_paths) = self.process_library_paths(config);
        let excluded = Arc::new(excluded_paths);
        let found = Arc::new(Mutex::new(HashSet::new()));

        // Escanea cada volumen en paralelo
        library_paths
            .par_iter()
            .filter(|base| base.is_dir())
            .for_each(|base| {
                let start = Instant::now();
                self.scan_base_dir(
                    base,
                    excluded.clone(),
                    found.clone(),
                    config.follow_symlinks,
                );
                println!("Scanned {} in {:?}", base.display(), start.elapsed());
            });

        // Extrae el conjunto final
        let flat: HashSet<PathBuf> = Arc::try_unwrap(found)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc_mutex| arc_mutex.lock().unwrap().clone());

        // Agrupa por unidad/disco
        let mut by_unit: HashMap<String, HashSet<PathBuf>> = HashMap::new();
        for path in flat {
            let unit = Self::root_of_path(&path);
            by_unit.entry(unit).or_default().insert(path);
        }

        by_unit
    }
}
