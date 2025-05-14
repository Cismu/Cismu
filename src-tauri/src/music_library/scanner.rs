use super::config::LibraryConfig;
use super::traits::Scanner;
use super::utils::AudioFormat;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Default)]
pub struct DefaultScanner;

impl DefaultScanner {
    fn process_library_paths(
        &self,
        config: &LibraryConfig,
    ) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
        // 1) Rutas a escanear según config
        let library_paths = config
            .scan_directories
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        // 2) Exclusiones iniciales desde config
        let mut excluded_paths = config
            .excluded_directories
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        // 3) Variables de entorno con defaults
        let env_vars_with_defaults = [
            ("WINDIR", "C:\\Windows"),
            ("ProgramFiles", "C:\\Program Files"),
            ("ProgramFiles(x86)", "C:\\Program Files (x86)"),
            ("ProgramData", "C:\\ProgramData"),
            ("LOCALAPPDATA", ""),
            ("APPDATA", ""),
            ("TEMP", ""),
        ];

        for &(var, default) in &env_vars_with_defaults {
            let val = env::var(var).unwrap_or_else(|_| default.to_string());
            if !val.is_empty() {
                excluded_paths.insert(PathBuf::from(val));
            }
        }

        // 4) Exclusiones relativas bajo cada raíz de library_paths
        let relative_exclusions = [
            "$Recycle.Bin",
            "System Volume Information",
            "Recovery",
            "PerfLogs",
        ];
        let mut seen_roots = HashSet::new();

        for lib in &library_paths {
            if let Some(root) = lib.ancestors().last().map(PathBuf::from) {
                if seen_roots.insert(root.clone()) {
                    for rel in &relative_exclusions {
                        excluded_paths.insert(root.join(rel));
                    }
                }
            }
        }

        (library_paths, excluded_paths)
    }
}

impl Scanner for DefaultScanner {
    fn scan(&self, config: &LibraryConfig) -> HashSet<PathBuf> {
        // Obtener rutas base y rutas excluidas:
        let (library_paths, excluded_paths) = self.process_library_paths(config);

        let mut found = HashSet::new();

        // Closure para filtrar entradas excluidas
        let is_not_excluded =
            |entry: &DirEntry| !excluded_paths.iter().any(|ex| entry.path().starts_with(ex));

        // Closure para aceptar sólo ficheros de audio
        let is_audio_file = |e: &DirEntry| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| AudioFormat::from_extension(ext))
                    .is_some()
        };

        for base in library_paths {
            if !base.is_dir() {
                continue;
            }

            WalkDir::new(&base)
                .follow_links(config.follow_symlinks)
                .into_iter()
                // Evitar entrar en rutas excluidas
                .filter_entry(|e| is_not_excluded(e))
                .filter_map(Result::ok)
                // Sólo archivos de audio
                .filter(is_audio_file)
                .for_each(|entry| {
                    found.insert(entry.into_path());
                });
        }

        found
    }
}
