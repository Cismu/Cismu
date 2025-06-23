use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock, mpsc::channel},
    thread::{self, JoinHandle},
};

use anyhow::Result;
use notify::{EventKind, RecursiveMode, Watcher, recommended_watcher};

use crate::error::ConfigError;
use crate::library_config::LibraryConfig;
use cismu_paths::PATHS;

/// Eventos emitidos tras cambios o errores
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Loaded(LibraryConfig),
    Updated(LibraryConfig),
    Error(Arc<ConfigError>),
}

/// Tipo de callback para suscriptores
type Subscriber = Box<dyn Fn(ConfigEvent) + Send + Sync>;
type SubsMap = Arc<Mutex<HashMap<usize, Subscriber>>>;

pub struct ConfigManager {
    path: PathBuf,
    // data: Arc<RwLock<LibraryConfig>>,
    // subscribers: SubsMap,
    // history: Arc<Mutex<Vec<LibraryConfig>>>,
    // watcher_handle: Option<JoinHandle<()>>,
    // next_sub_id: Arc<Mutex<usize>>,
}

impl ConfigManager {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        let initial = if path.exists() {
            LibraryConfig::from_file(&path)?
        } else {
            LibraryConfig::default()
        };

        Ok(Self { path })
    }
}

// // 2. Estructuras compartidas
// let data = Arc::new(RwLock::new(initial.clone()));
// let subscribers: SubsMap = Arc::new(Mutex::new(HashMap::new()));
// let history = Arc::new(Mutex::new(vec![initial.clone()]));
// let next_sub_id = Arc::new(Mutex::new(0));

// // 3. Clones para el hilo
// let d_cl = Arc::clone(&data);
// let s_cl = Arc::clone(&subscribers);
// let h_cl = Arc::clone(&history);
// let p_cl = path.clone();

// // 4. Hilo que vigila el fichero
// let watcher_handle = Some(thread::spawn(move || {
//     let (tx, rx) = channel::<notify::Result<notify::Event>>();
//     let mut watcher = recommended_watcher(move |res| {
//         let _ = tx.send(res);
//     })
//     .expect("no pudo crear watcher");
//     watcher
//         .watch(&p_cl, RecursiveMode::Recursive)
//         .expect("no pudo vigilar ruta");

//     for res in rx {
//         match res {
//             Ok(event) if matches!(event.kind, EventKind::Modify(_)) => {
//                 match LibraryConfig::from_file(&p_cl) {
//                     Ok(new_cfg) => {
//                         h_cl.lock().unwrap().push(new_cfg.clone());
//                         *d_cl.write().unwrap() = new_cfg.clone();
//                         for cb in s_cl.lock().unwrap().values() {
//                             cb(ConfigEvent::Updated(new_cfg.clone()));
//                         }
//                     }
//                     Err(e) => {
//                         let err_arc = Arc::new(e);
//                         for cb in s_cl.lock().unwrap().values() {
//                             cb(ConfigEvent::Error(err_arc.clone()));
//                         }
//                     }
//                 }
//             }
//             Err(err) => {
//                 let err_arc = Arc::new(ConfigError::Notify(err));
//                 for cb in s_cl.lock().unwrap().values() {
//                     cb(ConfigEvent::Error(err_arc.clone()));
//                 }
//             }
//             _ => {}
//         }
//     }
// }));

// Ok(Self {
//     path,
//     data,
//     subscribers,
//     history,
//     watcher_handle,
//     next_sub_id,
// })

// /// Devuelve una copia de la configuración actual
// pub fn get(&self) -> LibraryConfig {
//     self.data.read().unwrap().clone()
// }

// /// Aplica un closure, escribe el TOML y notifica
// pub fn update<F>(&self, updater: F) -> Result<(), ConfigError>
// where
//     F: FnOnce(&mut LibraryConfig),
// {
//     let mut cfg = self.data.write().unwrap();
//     updater(&mut cfg);
//     let toml_str = toml::to_string(&*cfg)?;
//     fs::write(&self.path, toml_str)?;
//     self.history.lock().unwrap().push(cfg.clone());
//     for cb in self.subscribers.lock().unwrap().values() {
//         cb(ConfigEvent::Updated(cfg.clone()));
//     }
//     Ok(())
// }

// /// Fuerza recarga desde disco y notifica
// pub fn refresh(&self) -> Result<(), ConfigError> {
//     let new_cfg = LibraryConfig::from_file(&self.path)?;
//     *self.data.write().unwrap() = new_cfg.clone();
//     for cb in self.subscribers.lock().unwrap().values() {
//         cb(ConfigEvent::Updated(new_cfg.clone()));
//     }
//     Ok(())
// }

// /// Deshace el último cambio (patrón Memento)
// pub fn rollback(&self) -> bool {
//     let mut hist = self.history.lock().unwrap();
//     if hist.len() > 1 {
//         hist.pop();
//         let prev = hist.last().unwrap().clone();
//         *self.data.write().unwrap() = prev.clone();
//         let _ = fs::write(&self.path, toml::to_string(&prev).unwrap());
//         for cb in self.subscribers.lock().unwrap().values() {
//             cb(ConfigEvent::Updated(prev.clone()));
//         }
//         true
//     } else {
//         false
//     }
// }

// /// Se suscribe a eventos, devuelve un ID
// pub fn subscribe<F>(&self, cb: F) -> usize
// where
//     F: Fn(ConfigEvent) + Send + Sync + 'static,
// {
//     let mut id_lock = self.next_sub_id.lock().unwrap();
//     let id = *id_lock;
//     *id_lock += 1;
//     self.subscribers.lock().unwrap().insert(id, Box::new(cb));
//     id
// }

// /// Cancela una suscripción
// pub fn unsubscribe(&self, id: usize) {
//     self.subscribers.lock().unwrap().remove(&id);
// }
