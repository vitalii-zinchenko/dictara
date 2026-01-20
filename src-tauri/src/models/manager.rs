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

    /// Get the path to a model, trying new structure first then falling back to old.
    ///
    /// New structure: models_dir/{model.name}/{file.filename}
    /// Old structure: models_dir/{model.filename} (single-file) or models_dir/{model.filename}/ (multi-file)
    ///
    /// Returns the directory path (not individual file path).
    fn get_model_path_with_fallback(&self, entry: &ModelCatalogEntry) -> PathBuf {
        let new_dir = self.models_dir.join(&entry.name);

        // Check if new structure exists
        if new_dir.exists() {
            return new_dir;
        }

        // Fall back to old structure
        if entry.files.len() == 1 {
            // Old single-file: models_dir/ggml-small.bin (file, not directory)
            // But we need to return directory for consistency, check if old file exists
            let old_file_path = self.models_dir.join(&entry.filename);
            if old_file_path.exists() {
                // Return the parent directory (models_dir) - caller will handle file vs dir
                return self.models_dir.clone();
            }
        } else {
            // Old multi-file: models_dir/parakeet-tdt-0.6b-v3-int8/ (directory)
            let old_dir = self.models_dir.join(&entry.filename);
            if old_dir.exists() {
                return old_dir;
            }
        }

        // Neither exists, return new structure path (for new downloads)
        new_dir
    }

    /// Migrate old single-file models to new unified directory structure.
    ///
    /// Runs asynchronously on app startup. Moves:
    /// - Old: models_dir/ggml-small.bin
    /// - New: models_dir/whisper-small/ggml-small.bin
    ///
    /// Multi-file models (Parakeet) don't need migration as they already use directory structure.
    pub async fn migrate_old_models(&self) -> Result<(), String> {
        info!("Checking for models to migrate to new structure...");

        for entry in get_model_catalog() {
            // Only single-file models need migration
            if entry.files.len() == 1 {
                let old_file_path = self.models_dir.join(&entry.filename);
                let new_dir = self.models_dir.join(&entry.name);
                let new_file_path = new_dir.join(&entry.files[0].filename);

                // Migrate if old exists and new doesn't
                if old_file_path.exists() && !new_file_path.exists() {
                    tokio::fs::create_dir_all(&new_dir).await.map_err(|e| {
                        format!("Failed to create directory during migration: {}", e)
                    })?;

                    tokio::fs::rename(&old_file_path, &new_file_path)
                        .await
                        .map_err(|e| format!("Failed to migrate {}: {}", entry.name, e))?;

                    info!(
                        "Migrated {} to new structure: {:?}",
                        entry.name, new_file_path
                    );

                    // Also migrate .partial file if exists
                    let old_partial = self.models_dir.join(format!("{}.partial", entry.filename));
                    if old_partial.exists() {
                        let new_partial =
                            new_dir.join(format!("{}.partial", entry.files[0].filename));
                        let _ = tokio::fs::rename(&old_partial, &new_partial).await;
                        debug!("Migrated partial file for {}", entry.name);
                    }
                }
            }
        }

        info!("Model migration check complete");
        Ok(())
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
    /// Checks both new unified structure and old structure for backward compatibility.
    fn compute_status(&self, entry: &ModelCatalogEntry, loader: &ModelLoader) -> ModelStatus {
        let is_downloading = self
            .downloading
            .lock()
            .unwrap()
            .get(&entry.name)
            .copied()
            .unwrap_or(false);

        // Check if downloaded in new structure: models_dir/{name}/{files}
        let new_dir = self.models_dir.join(&entry.name);
        let is_downloaded_new = new_dir.is_dir()
            && entry
                .files
                .iter()
                .all(|f| new_dir.join(&f.filename).exists());

        // Check if downloaded in old structure
        let is_downloaded_old = if entry.files.len() == 1 {
            // Old single-file: models_dir/{filename}
            self.models_dir.join(&entry.filename).exists()
        } else {
            // Old multi-file: models_dir/{filename}/ (same as new for current Parakeet models)
            let old_dir = self.models_dir.join(&entry.filename);
            old_dir.is_dir()
                && entry
                    .files
                    .iter()
                    .all(|f| old_dir.join(&f.filename).exists())
        };

        let is_downloaded = is_downloaded_new || is_downloaded_old;

        // Track partial download progress (aggregate across all files)
        let downloaded_bytes = {
            let mut total = 0u64;
            for file in &entry.files {
                // Check new location first
                let new_partial = new_dir.join(format!("{}.partial", file.filename));
                if new_partial.exists() {
                    total += new_partial.metadata().map(|m| m.len()).unwrap_or(0);
                } else if entry.files.len() == 1 {
                    // Check old location for single-file models
                    let old_partial = self.models_dir.join(format!("{}.partial", entry.filename));
                    total += old_partial.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
            total
        };

        ModelStatus {
            is_downloaded,
            is_downloading,
            is_loaded: loader.is_model_loaded(&entry.name),
            is_loading: loader.is_model_loading(&entry.name),
            downloaded_bytes,
        }
    }

    /// Start downloading a model.
    ///
    /// Emits progress events to the frontend during download.
    /// Downloads into new unified structure: models_dir/{model_name}/{files}
    /// Supports resuming interrupted downloads with .partial files.
    pub async fn download_model(&self, model_name: &str, app: AppHandle) -> Result<(), String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        // Check if already downloaded (check both new and old structures)
        let new_dir = self.models_dir.join(&entry.name);
        let is_downloaded_new = new_dir.is_dir()
            && entry
                .files
                .iter()
                .all(|f| new_dir.join(&f.filename).exists());

        let is_downloaded_old = if entry.files.len() == 1 {
            self.models_dir.join(&entry.filename).exists()
        } else {
            let old_dir = self.models_dir.join(&entry.filename);
            old_dir.is_dir()
                && entry
                    .files
                    .iter()
                    .all(|f| old_dir.join(&f.filename).exists())
        };

        if is_downloaded_new || is_downloaded_old {
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

        // Use unified download for all models (always use new structure)
        let result = self
            .download_model_unified(&entry, &app, &cancel_token)
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

    /// Unified download implementation for all models (single-file and multi-file).
    ///
    /// Downloads all files in parallel, aggregates progress, and verifies checksums in parallel.
    /// Uses new structure: models_dir/{model_name}/{files}
    async fn download_model_unified(
        &self,
        entry: &ModelCatalogEntry,
        app: &AppHandle,
        cancel_token: &CancellationToken,
    ) -> Result<(), String> {
        let model_dir = self.models_dir.join(&entry.name);

        // Create model directory
        tokio::fs::create_dir_all(&model_dir)
            .await
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        let total_size = entry.size_bytes;
        let file_count = entry.files.len();

        // Shared progress state: tracks downloaded bytes per file
        let progress = Arc::new(Mutex::new(vec![0u64; file_count]));

        info!(
            "Downloading model '{}' ({} files) into {:?}",
            entry.name, file_count, model_dir
        );

        // Phase 1: Download all files in parallel
        let mut download_handles = vec![];

        for (idx, file) in entry.files.iter().enumerate() {
            let file = file.clone();
            let model_dir = model_dir.clone();
            let cancel_token = cancel_token.clone();
            let progress = progress.clone();
            let app = app.clone();
            let model_name = entry.name.clone();

            let handle = tokio::spawn(async move {
                let partial_path = model_dir.join(format!("{}.partial", file.filename));

                // Check for existing partial download to resume
                let resume_from = if partial_path.exists() {
                    partial_path.metadata().map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };

                if resume_from > 0 {
                    info!("Resuming {} from {} bytes", file.filename, resume_from);
                }

                // Download with progress tracking and resume support
                Self::download_file_with_progress(
                    &file.url,
                    &partial_path,
                    resume_from,
                    &cancel_token,
                    idx,
                    &progress,
                    total_size,
                    &model_name,
                    &app,
                )
                .await
            });

            download_handles.push(handle);
        }

        // Wait for all downloads to complete
        for (idx, handle) in download_handles.into_iter().enumerate() {
            handle
                .await
                .map_err(|e| format!("Download task {} failed: {}", idx, e))??;
        }

        // Check for cancellation before verification
        if cancel_token.is_cancelled() {
            // Clean up partial downloads
            let _ = tokio::fs::remove_dir_all(&model_dir).await;
            return Err("Download cancelled".to_string());
        }

        info!("All files downloaded, verifying checksums...");

        // Emit verifying state
        let _ = ModelDownloadStateChanged::Verifying {
            model_name: entry.name.clone(),
        }
        .emit(app);

        // Phase 2: Verify checksums in parallel
        let mut verify_handles = vec![];

        for file in &entry.files {
            let file = file.clone();
            let model_dir = model_dir.clone();

            let handle = tokio::spawn(async move {
                let partial_path = model_dir.join(format!("{}.partial", file.filename));

                // Skip verification for "TBD" checksums (temporary during development)
                if file.sha256 != "TBD" {
                    Self::verify_checksum(&partial_path, &file.sha256).await
                } else {
                    Ok(())
                }
            });

            verify_handles.push(handle);
        }

        // Wait for all verifications
        for (idx, handle) in verify_handles.into_iter().enumerate() {
            if let Err(e) = handle
                .await
                .map_err(|e| format!("Verification task {} failed: {}", idx, e))?
            {
                // Clean up on verification failure
                let _ = tokio::fs::remove_dir_all(&model_dir).await;
                return Err(format!("Checksum verification failed: {}", e));
            }
        }

        info!("All checksums verified, finalizing...");

        // Phase 3: Rename all .partial files to final names
        for file in &entry.files {
            let file_path = model_dir.join(&file.filename);
            let partial_path = model_dir.join(format!("{}.partial", file.filename));

            tokio::fs::rename(&partial_path, &file_path)
                .await
                .map_err(|e| format!("Failed to rename {} to final: {}", file.filename, e))?;
        }

        info!(
            "Model '{}' download complete: {} files",
            entry.name, file_count
        );

        Ok(())
    }

    /// Download a single file with progress tracking and resume support.
    ///
    /// Updates shared progress state and emits progress events aggregated across all files.
    #[allow(clippy::too_many_arguments)]
    async fn download_file_with_progress(
        url: &str,
        dest_path: &Path,
        resume_from: u64,
        cancel_token: &CancellationToken,
        file_index: usize,
        progress: &Arc<Mutex<Vec<u64>>>,
        total_size: u64,
        model_name: &str,
        app: &AppHandle,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();

        // Build request with Range header for resume
        let mut request = client.get(url);
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

        // Open file for appending (if resuming) or create new
        let file = if resume_from > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(dest_path)
                .await
                .map_err(|e| format!("Failed to open file for append: {}", e))?
        } else {
            tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(dest_path)
                .await
                .map_err(|e| format!("Failed to create file: {}", e))?
        };

        let mut file = tokio::io::BufWriter::new(file);
        let mut stream = response.bytes_stream();
        let mut downloaded_this_session = 0u64;
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            if cancel_token.is_cancelled() {
                return Err("Download cancelled".to_string());
            }

            let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            downloaded_this_session += chunk.len() as u64;

            // Update shared progress and emit every 100ms to avoid flooding
            if last_emit.elapsed().as_millis() >= 100 {
                let mut prog = progress.lock().unwrap();
                prog[file_index] = resume_from + downloaded_this_session;
                let total_downloaded: u64 = prog.iter().sum();
                drop(prog);

                let percentage = (total_downloaded as f64 / total_size as f64) * 100.0;

                debug!(
                    "Download progress: {:.1}% ({}/{} bytes)",
                    percentage, total_downloaded, total_size
                );

                let _ = ModelDownloadStateChanged::Progress {
                    model_name: model_name.to_string(),
                    downloaded_bytes: total_downloaded,
                    total_bytes: total_size,
                    percentage,
                }
                .emit(app);

                last_emit = std::time::Instant::now();
            }
        }

        // Final progress update for this file
        {
            let mut prog = progress.lock().unwrap();
            prog[file_index] = resume_from + downloaded_this_session;
        }

        // Flush file
        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file: {}", e))?;

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
    /// Removes model from both new and old locations if they exist.
    pub fn delete_model(&self, model_name: &str, loader: &ModelLoader) -> Result<(), String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        // Unload if currently loaded
        if loader.is_model_loaded(model_name) {
            loader.unload_model();
        }

        let mut deleted_something = false;

        // Delete from new structure: models_dir/{name}/
        let new_dir = self.models_dir.join(&entry.name);
        if new_dir.exists() && new_dir.is_dir() {
            std::fs::remove_dir_all(&new_dir)
                .map_err(|e| format!("Failed to delete model directory: {}", e))?;
            info!("Deleted model from new location: {:?}", new_dir);
            deleted_something = true;
        }

        // Delete from old structure if it exists
        if entry.files.len() == 1 {
            // Old single-file: models_dir/{filename}
            let old_file = self.models_dir.join(&entry.filename);
            if old_file.exists() {
                std::fs::remove_file(&old_file)
                    .map_err(|e| format!("Failed to delete old model file: {}", e))?;
                info!("Deleted model from old location: {:?}", old_file);
                deleted_something = true;
            }

            // Also delete old .partial if exists
            let old_partial = self.models_dir.join(format!("{}.partial", entry.filename));
            if old_partial.exists() {
                let _ = std::fs::remove_file(&old_partial);
            }
        } else {
            // Old multi-file: models_dir/{filename}/ (might be same as new for Parakeet)
            let old_dir = self.models_dir.join(&entry.filename);
            if old_dir.exists() && old_dir.is_dir() && old_dir != new_dir {
                std::fs::remove_dir_all(&old_dir)
                    .map_err(|e| format!("Failed to delete old model directory: {}", e))?;
                info!("Deleted model from old location: {:?}", old_dir);
                deleted_something = true;
            }
        }

        if !deleted_something {
            warn!("Model '{}' was not found in any location", model_name);
        }

        Ok(())
    }

    /// Get the path to a model directory.
    /// Returns new structure first, falls back to old structure for backward compatibility.
    ///
    /// New structure: models_dir/{model_name}/
    /// Old structure: models_dir/{filename} (single) or models_dir/{filename}/ (multi)
    pub fn get_model_path(&self, model_name: &str) -> Result<PathBuf, String> {
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        Ok(self.get_model_path_with_fallback(&entry))
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
