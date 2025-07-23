use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tracing::{error, info};

use futures::{StreamExt, stream::FuturesUnordered};

use crate::{
    audio_analysis::fingerprint, config_manager::ConfigManager, enrichment::acoustid::AcoustidClient,
    parsing::LocalMetadata, scanning::LocalScanner, storage::LocalStorage,
};

#[derive(Debug, Clone)]
pub struct LibraryManager {
    scanner: Arc<LocalScanner>,
    metadata: Arc<LocalMetadata>,
    storage: Arc<LocalStorage>,
}

impl LibraryManager {
    pub fn new(config: ConfigManager) -> Self {
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage).unwrap();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            storage: Arc::new(storage),
        }
    }

    pub async fn scan(&self) -> Result<()> {
        let scanner = Arc::clone(&self.scanner);
        let metadata = Arc::clone(&self.metadata);
        let storage = Arc::clone(&self.storage);

        let groups = scanner.scan().await?;
        let mut rx = metadata.process(groups);

        while let Some(res) = rx.recv().await {
            match res {
                Ok(track) => {
                    storage.resolve_unresolved_track(track)?;
                }
                Err(e) => {
                    error!(%e, "error al procesar metadata");
                }
            }
        }

        self.process_fingerprint_queue().await?;
        self.process_verification_queue().await?;

        Ok(())
    }

    /// Procesa la cola de pistas pendientes de fingerprinting.
    async fn process_fingerprint_queue(&self) -> Result<()> {
        loop {
            let queue = self.storage.get_fingerprint_queue(10)?;
            if queue.is_empty() {
                info!("Cola de fingerprints vacía. Proceso finalizado.");
                break;
            }

            let mut futs = FuturesUnordered::new();
            for (track_id, path) in queue {
                let storage = self.storage.clone();
                futs.push(tokio::spawn(async move {
                        info!("Generando fingerprint para: {}", path.display());
                        match fingerprint::fingerprint_from_file(&path) {
                            Ok(fp) => {
                                if let Err(e) = storage.set_fingerprint_for_track(track_id, &fp) {
                                    error!(error = %e, "Error al guardar fingerprint para {}", path.display());
                                }
                            }
                            Err(e) => {
                                 error!(error = %e, "Error al generar fingerprint para {}", path.display());
                            }
                        }
                    }));
            }

            while futs.next().await.is_some() {}
        }

        Ok(())
    }

    /// Procesa la cola de pistas pendientes de fingerprinting.
    async fn process_verification_queue(&self) -> Result<()> {
        let acoustid_client = Arc::new(AcoustidClient::new("atU38bBrIw"));

        loop {
            let queue = self.storage.get_verification_queue(3)?;
            if queue.is_empty() {
                info!("Cola de verificación vacía. Proceso finalizado.");
                break;
            }

            info!(
                "Procesando un lote de {} pistas para verificación online...",
                queue.len()
            );

            let mut futs = FuturesUnordered::new();
            for (track_id, path, duration) in queue {
                let storage = self.storage.clone();
                let client = acoustid_client.clone();
                futs.push(tokio::spawn(async move {
                    if let Ok(fp) = fingerprint::fingerprint_from_file(&path) {
                        match client.lookup(&fp, duration.as_secs() as u32).await {
                            Ok(results) => {
                                if let Some(best_result) = results.iter().max_by(|a, b| a.score.total_cmp(&b.score)) {
                                    info!("Coincidencia encontrada para {}: AcoustID {}", path.display(), best_result.id);
                                    if let Err(e) = storage.update_track_with_acoustid(track_id, best_result) {
                                        error!(error = %e, "Error al actualizar la DB con datos de AcoustID");
                                    }
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Error en la llamada a la API de AcoustID para {}", path.display());
                            }
                        }
                    }
                }));
            }

            // Esperamos que el lote termine
            while futs.next().await.is_some() {}

            // Respetamos el rate limit
            info!("Lote de verificación procesado. Esperando 4 segundos...");
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
        Ok(())
    }
}

impl Default for LibraryManager {
    fn default() -> Self {
        let config = ConfigManager::default();
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage).unwrap();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            storage: Arc::new(storage),
        }
    }
}

// API Key for submit fingerprint to AcoustID: 2igXkDlTE9
// API Key for use AcoustID: atU38bBrIw
