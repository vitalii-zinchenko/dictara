use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use super::catalog::{get_model_catalog, ModelCatalogEntry};
use super::events::ModelDownloadStateChanged;
use super::loader::ModelLoader;
use super::status::ModelStatus;
use super::ModelInfo;

/// Manages model downloads, storage, and status tracking.
/// Does NOT handle model loading into memory - that's ModelLoader's job.
pub struct ModelManager {
    models_dir: PathBuf,
    /// In-memory state tracking which models are currently downloading
    downloading: Arc<Mutex<HashMap<String, bool>>>,
    /// Cancellation tokens for active downloads
    cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl ModelManager {
    /// Create a new ModelManager.
    ///
    /// # Arguments
    /// * `app` - Tauri app handle to get the app data directory
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        let models_dir = app_data_dir.join("models");

        // Create models directory if it doesn't exist
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;

        info!("Models directory: {:?}", models_dir);

        Ok(Self {
            models_dir,
            downloading: Arc::new(Mutex::new(HashMap::new())),
            cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the models directory path
    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    /// Get all models with their current status.
    pub fn get_all_models(&self, loader: &ModelLoader) -> Vec<ModelInfo> {
        get_model_catalog()
            .iter()
            .map(|entry| {
                let status = self.compute_status(entry, loader);
                ModelInfo::from_catalog_and_status(entry, &status)
            })
            .collect()
    }

    /// Compute runtime status for a model.
    fn compute_status(&self, entry: &ModelCatalogEntry, loader: &ModelLoader) -> ModelStatus {
        let model_path = self.models_dir.join(&entry.filename);
        let partial_path = self.models_dir.join(format!("{}.partial", entry.filename));

        let is_downloading = self
            .downloading
            .lock()
            .unwrap()
            .get(&entry.name)
            .copied()
            .unwrap_or(false);

        ModelStatus {
            is_downloaded: model_path.exists(),
            is_downloading,
            is_loaded: loader.is_model_loaded(&entry.name),
            is_loading: loader.is_model_loading(&entry.name),
            downloaded_bytes: partial_path.metadata().map(|m| m.len()).unwrap_or(0),
        }
    }

    /// Start downloading a model.
    ///
    /// Emits progress events to the frontend during download.
    /// Supports resuming interrupted downloads.
    pub async fn download_model(&self, model_name: &str, app: AppHandle) -> Result<(), String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        let model_path = self.models_dir.join(&entry.filename);
        let partial_path = self.models_dir.join(format!("{}.partial", entry.filename));

        // Check if already downloaded
        if model_path.exists() {
            info!("Model '{}' already downloaded", model_name);
            return Ok(());
        }

        // Check if already downloading
        {
            let downloading = self.downloading.lock().unwrap();
            if downloading.get(model_name).copied().unwrap_or(false) {
                warn!("Model '{}' is already being downloaded", model_name);
                return Err(format!(
                    "Model '{}' is already being downloaded",
                    model_name
                ));
            }
        }

        // Mark as downloading
        {
            let mut downloading = self.downloading.lock().unwrap();
            downloading.insert(model_name.to_string(), true);
        }

        // Create cancellation token
        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.cancel_tokens.lock().unwrap();
            tokens.insert(model_name.to_string(), cancel_token.clone());
        }

        info!("Starting download of model '{}'", model_name);

        // Check for partial download to resume
        let resume_from = if partial_path.exists() {
            let size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
            info!("Resuming download from {} bytes", size);
            size
        } else {
            0
        };

        // Perform download
        let result = self
            .do_download(
                &entry,
                &partial_path,
                &model_path,
                resume_from,
                &app,
                &cancel_token,
            )
            .await;

        // Clear downloading state
        {
            let mut downloading = self.downloading.lock().unwrap();
            downloading.remove(model_name);
        }
        {
            let mut tokens = self.cancel_tokens.lock().unwrap();
            tokens.remove(model_name);
        }

        match &result {
            Ok(()) => {
                info!("Download complete: {}", model_name);
                let _ = ModelDownloadStateChanged::Complete {
                    model_name: model_name.to_string(),
                }
                .emit(&app);
            }
            Err(e) => {
                error!("Download failed: {}", e);
                let _ = ModelDownloadStateChanged::Error {
                    model_name: model_name.to_string(),
                    error: e.to_string(),
                }
                .emit(&app);
            }
        }

        result
    }

    /// Perform the actual download.
    async fn do_download(
        &self,
        entry: &ModelCatalogEntry,
        partial_path: &PathBuf,
        final_path: &PathBuf,
        resume_from: u64,
        app: &AppHandle,
        cancel_token: &CancellationToken,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();

        // Build request with Range header for resume
        let mut request = client.get(&entry.url);
        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;

        // Check response status
        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        // Get total size from Content-Length or Content-Range
        let total_bytes = if resume_from > 0 {
            // For resumed downloads, parse Content-Range header
            response
                .headers()
                .get("content-range")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.split('/').next_back())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(entry.estimated_size_bytes)
        } else {
            response
                .content_length()
                .unwrap_or(entry.estimated_size_bytes)
        };

        // Open file for appending (or create)
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(partial_path)
            .await
            .map_err(|e| format!("Failed to create partial file: {}", e))?;

        let mut file = tokio::io::BufWriter::new(file);
        let mut stream = response.bytes_stream();
        let mut downloaded = resume_from;
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if cancel_token.is_cancelled() {
                info!("Download cancelled");
                return Err("Download cancelled".to_string());
            }

            let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            downloaded += chunk.len() as u64;

            // Emit progress every 100ms to avoid flooding
            if last_emit.elapsed().as_millis() >= 100 {
                let percentage = (downloaded as f64 / total_bytes as f64) * 100.0;

                debug!(
                    "Download progress: {:.1}% ({}/{})",
                    percentage, downloaded, total_bytes
                );

                let _ = ModelDownloadStateChanged::Progress {
                    model_name: entry.name.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    percentage,
                }
                .emit(app);
                last_emit = std::time::Instant::now();
            }
        }

        // Flush and close file
        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file: {}", e))?;

        // Drop the file handle to ensure it's closed before verification
        drop(file);

        // Emit verifying event and verify checksum
        info!("Verifying checksum for model '{}'", entry.name);
        let _ = ModelDownloadStateChanged::Verifying {
            model_name: entry.name.clone(),
        }
        .emit(app);

        if let Err(e) = Self::verify_checksum(partial_path, &entry.sha256).await {
            // Delete the corrupted partial file
            let _ = tokio::fs::remove_file(partial_path).await;
            return Err(format!("Checksum verification failed: {}", e));
        }
        info!("Checksum verified successfully for model '{}'", entry.name);

        // Rename partial to final
        tokio::fs::rename(partial_path, final_path)
            .await
            .map_err(|e| format!("Failed to rename partial file: {}", e))?;

        Ok(())
    }

    /// Verify SHA-256 checksum of a downloaded file.
    async fn verify_checksum(file_path: &Path, expected_hash: &str) -> Result<(), String> {
        let mut file = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| format!("Failed to open file for verification: {}", e))?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(|e| format!("Failed to read file for verification: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != expected_hash {
            return Err(format!(
                "Hash mismatch: expected {}, got {}",
                expected_hash, computed_hash
            ));
        }

        Ok(())
    }

    /// Cancel an ongoing download.
    pub fn cancel_download(&self, model_name: &str) -> Result<(), String> {
        let tokens = self.cancel_tokens.lock().unwrap();
        if let Some(token) = tokens.get(model_name) {
            token.cancel();
            info!("Cancellation requested for model '{}'", model_name);
            Ok(())
        } else {
            Err(format!("No active download for model '{}'", model_name))
        }
    }

    /// Delete a downloaded model.
    pub fn delete_model(&self, model_name: &str, loader: &ModelLoader) -> Result<(), String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        let model_path = self.models_dir.join(&entry.filename);
        let partial_path = self.models_dir.join(format!("{}.partial", entry.filename));

        // Unload if currently loaded
        if loader.is_model_loaded(model_name) {
            loader.unload_model();
        }

        // Delete model file
        if model_path.exists() {
            std::fs::remove_file(&model_path)
                .map_err(|e| format!("Failed to delete model file: {}", e))?;
            info!("Deleted model file: {:?}", model_path);
        }

        // Delete partial file if exists
        if partial_path.exists() {
            std::fs::remove_file(&partial_path)
                .map_err(|e| format!("Failed to delete partial file: {}", e))?;
        }

        Ok(())
    }

    /// Get the path to a model file.
    pub fn get_model_path(&self, model_name: &str) -> Result<PathBuf, String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        Ok(self.models_dir.join(&entry.filename))
    }

    /// Check if a model is downloaded.
    pub fn is_model_downloaded(&self, model_name: &str) -> bool {
        if let Ok(path) = self.get_model_path(model_name) {
            path.exists()
        } else {
            false
        }
    }
}
