use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use tauri::AppHandle;
use tauri_specta::Event;

use super::catalog::get_model_catalog;
use super::events::ModelLoadingStateChanged;
use super::local_client::LocalClient;

/// Loaded model state
struct LoadedModel {
    name: String,
    client: LocalClient,
}

/// Manages the currently loaded Whisper model in memory.
///
/// Only one model can be loaded at a time. Loading a new model
/// automatically unloads the previous one.
pub struct ModelLoader {
    current_model: Arc<Mutex<Option<LoadedModel>>>,
    loading: Arc<Mutex<Option<String>>>,
    models_dir: PathBuf,
}

impl ModelLoader {
    /// Create a new ModelLoader.
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            current_model: Arc::new(Mutex::new(None)),
            loading: Arc::new(Mutex::new(None)),
            models_dir,
        }
    }

    /// Resolve model path with backward compatibility.
    ///
    /// New structure: models_dir/{model_name}/{files}
    /// Old structure: models_dir/{filename} (single) or models_dir/{filename}/ (multi)
    ///
    /// Returns the path that LocalClient::new expects:
    /// - For single-file (Whisper): path to the .bin file
    /// - For multi-file (Parakeet): path to the directory containing model files
    fn resolve_model_path(&self, entry: &super::catalog::ModelCatalogEntry) -> PathBuf {
        // New structure
        let new_dir = self.models_dir.join(&entry.name);

        if entry.files.len() == 1 {
            // Single-file model (Whisper)
            let new_file = new_dir.join(&entry.files[0].filename);
            if new_file.exists() {
                return new_file; // Return file path for LocalClient::new
            }

            // Fall back to old structure
            let old_file = self.models_dir.join(&entry.filename);
            if old_file.exists() {
                return old_file;
            }

            // Return new path for error messages
            new_file
        } else {
            // Multi-file model (Parakeet)
            if new_dir.is_dir() {
                return new_dir; // Return directory path for LocalClient::new
            }

            // Fall back to old structure (might be same as new for current Parakeet models)
            let old_dir = self.models_dir.join(&entry.filename);
            if old_dir.is_dir() {
                return old_dir;
            }

            // Return new path for error messages
            new_dir
        }
    }

    /// Load a model into memory.
    ///
    /// This is an async operation that:
    /// 1. Emits `model-loading-started` event
    /// 2. Unloads any currently loaded model
    /// 3. Loads the new model in a blocking task
    /// 4. Emits `model-loading-complete` or `model-loading-error` event
    pub async fn load_model(&self, model_name: &str, app: &AppHandle) -> Result<(), String> {
        // Check if already loaded
        if self.is_model_loaded(model_name) {
            info!("Model '{}' is already loaded", model_name);
            return Ok(());
        }

        // Check if already loading
        {
            let loading = self.loading.lock().unwrap();
            if loading.is_some() {
                return Err("Another model is currently loading".to_string());
            }
        }

        // Mark as loading
        {
            let mut loading = self.loading.lock().unwrap();
            *loading = Some(model_name.to_string());
        }

        // Emit loading started
        let _ = ModelLoadingStateChanged::Started {
            model_name: model_name.to_string(),
        }
        .emit(app);
        debug!(
            ">>> LOAD START: Model '{}' - beginning async load",
            model_name
        );
        info!("Loading model '{}'", model_name);

        // Look up model in catalog
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        let model_path = self.resolve_model_path(&entry);
        debug!("Model path resolved: {:?}", model_path);

        // Verify model exists
        if !model_path.exists() {
            let mut loading = self.loading.lock().unwrap();
            *loading = None;

            let error = format!("Model file not found: {:?}", model_path);
            let _ = ModelLoadingStateChanged::Error {
                model_name: model_name.to_string(),
                error: error.clone(),
            }
            .emit(app);
            return Err(error);
        }

        // Unload current model first
        self.unload_model();

        // Load model in blocking task (model loading is CPU-intensive)
        let model_name_clone = model_name.to_string();
        let model_type = entry.model_type;
        let result =
            tokio::task::spawn_blocking(move || LocalClient::new(&model_path, model_type)).await;

        // Clear loading state
        {
            let mut loading = self.loading.lock().unwrap();
            *loading = None;
        }

        match result {
            Ok(Ok(client)) => {
                // Store loaded model
                let mut current = self.current_model.lock().unwrap();
                *current = Some(LoadedModel {
                    name: model_name_clone.clone(),
                    client,
                });

                debug!(
                    "<<< LOAD COMPLETE: Model '{}' - now in memory and ready",
                    model_name_clone
                );
                info!("Model '{}' loaded successfully", model_name_clone);
                let _ = ModelLoadingStateChanged::Complete {
                    model_name: model_name_clone,
                }
                .emit(app);
                Ok(())
            }
            Ok(Err(e)) => {
                let error = format!("Failed to load model: {}", e);
                error!("{}", error);
                let _ = ModelLoadingStateChanged::Error {
                    model_name: model_name.to_string(),
                    error: error.clone(),
                }
                .emit(app);
                Err(error)
            }
            Err(e) => {
                let error = format!("Task panicked while loading model: {}", e);
                error!("{}", error);
                let _ = ModelLoadingStateChanged::Error {
                    model_name: model_name.to_string(),
                    error: error.clone(),
                }
                .emit(app);
                Err(error)
            }
        }
    }

    /// Unload the currently loaded model (frees memory).
    pub fn unload_model(&self) {
        let mut current = self.current_model.lock().unwrap();
        if let Some(model) = current.take() {
            debug!(
                ">>> UNLOAD START: Model '{}' - releasing memory",
                model.name
            );
            info!("Unloading model '{}'", model.name);
            // Model is dropped here, freeing memory
            debug!("<<< UNLOAD COMPLETE: Model '{}' - memory freed", model.name);
        } else {
            debug!("No model loaded to unload");
        }
    }

    /// Check if a specific model is currently loaded.
    pub fn is_model_loaded(&self, model_name: &str) -> bool {
        self.current_model
            .lock()
            .unwrap()
            .as_ref()
            .map(|m| m.name == model_name)
            .unwrap_or(false)
    }

    /// Check if a specific model is currently being loaded.
    pub fn is_model_loading(&self, model_name: &str) -> bool {
        self.loading
            .lock()
            .unwrap()
            .as_ref()
            .map(|name| name == model_name)
            .unwrap_or(false)
    }

    /// Check if any model is currently loaded.
    /// Note: Prefer `transcribe_with_model` which handles model verification atomically.
    #[allow(dead_code)]
    pub fn has_loaded_model(&self) -> bool {
        self.current_model.lock().unwrap().is_some()
    }

    /// Get the name of the currently loaded model.
    pub fn get_loaded_model_name(&self) -> Option<String> {
        self.current_model
            .lock()
            .unwrap()
            .as_ref()
            .map(|m| m.name.clone())
    }

    /// Transcribe audio using the currently loaded model.
    ///
    /// Returns an error if no model is loaded.
    /// Note: Prefer `transcribe_with_model` which verifies the correct model is loaded.
    #[allow(dead_code)]
    pub fn transcribe(&self, audio_path: &std::path::Path) -> Result<String, String> {
        let mut current = self.current_model.lock().unwrap();
        match current.as_mut() {
            Some(model) => model
                .client
                .transcribe_file(audio_path)
                .map_err(|e| e.to_string()),
            None => {
                warn!("Attempted to transcribe without a loaded model");
                Err("No model loaded".to_string())
            }
        }
    }

    /// Transcribe audio ensuring a specific model is loaded.
    ///
    /// This method handles the race condition where the model could be unloaded
    /// or swapped between checking and transcribing. It:
    /// 1. Checks if the correct model is loaded
    /// 2. Loads it if needed (or if a different model is loaded)
    /// 3. Verifies the model is still correct before transcribing
    ///
    /// # Arguments
    /// * `model_name` - The expected model name to use for transcription
    /// * `audio_path` - Path to the audio file
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(String)` - Error if loading failed or model mismatch detected
    pub fn transcribe_with_model(
        &self,
        model_name: &str,
        audio_path: &std::path::Path,
    ) -> Result<String, String> {
        // First, check if the correct model is loaded (without holding lock long)
        let needs_load = {
            let current = self.current_model.lock().unwrap();
            match current.as_ref() {
                Some(m) if m.name == model_name => false, // correct model loaded
                _ => true,                                // wrong model or no model
            }
        };

        if needs_load {
            info!(
                "Model '{}' not loaded, loading on-demand for transcription",
                model_name
            );
            self.load_model_sync(model_name)?;
        }

        // Now transcribe - recheck that model is still correct to handle race conditions
        let mut current = self.current_model.lock().unwrap();
        match current.as_mut() {
            Some(model) if model.name == model_name => {
                debug!("Transcribing with verified model '{}'", model_name);
                model
                    .client
                    .transcribe_file(audio_path)
                    .map_err(|e| e.to_string())
            }
            Some(model) => {
                // Race condition: another thread loaded a different model
                warn!(
                    "Model mismatch detected: expected '{}' but '{}' is loaded",
                    model_name, model.name
                );
                Err(format!(
                    "Model changed during transcription: expected '{}' but '{}' is loaded. Please try again.",
                    model_name, model.name
                ))
            }
            None => {
                // Race condition: model was unloaded after we loaded it
                warn!("Model was unloaded during transcription setup");
                Err("Model was unloaded during transcription. Please try again.".to_string())
            }
        }
    }

    /// Load a model synchronously (blocking).
    ///
    /// Used for on-demand loading during transcription when the model
    /// isn't already loaded. Unlike `load_model`, this doesn't emit events
    /// since the user is already waiting for transcription to complete.
    pub fn load_model_sync(&self, model_name: &str) -> Result<(), String> {
        // Check if already loaded
        if self.is_model_loaded(model_name) {
            info!("Model '{}' is already loaded", model_name);
            return Ok(());
        }

        // Check if already loading (shouldn't happen in sync context, but be safe)
        {
            let loading = self.loading.lock().unwrap();
            if loading.is_some() {
                return Err("Another model is currently loading".to_string());
            }
        }

        // Mark as loading
        {
            let mut loading = self.loading.lock().unwrap();
            *loading = Some(model_name.to_string());
        }

        debug!(
            ">>> LOAD START (sync): Model '{}' - beginning blocking load",
            model_name
        );
        info!(
            "Loading model '{}' synchronously for transcription",
            model_name
        );

        // Look up model in catalog
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or_else(|| format!("Model '{}' not found in catalog", model_name))?;

        let model_path = self.resolve_model_path(&entry);
        debug!("Model path resolved (sync): {:?}", model_path);

        // Verify model exists
        if !model_path.exists() {
            let mut loading = self.loading.lock().unwrap();
            *loading = None;
            return Err(format!("Model file not found: {:?}", model_path));
        }

        // Unload current model first
        self.unload_model();

        // Load model (blocking)
        let model_type = entry.model_type;
        debug!("Starting model initialization ({:?})...", model_type);
        let result = LocalClient::new(&model_path, model_type);

        // Clear loading state
        {
            let mut loading = self.loading.lock().unwrap();
            *loading = None;
        }

        match result {
            Ok(client) => {
                // Store loaded model
                let mut current = self.current_model.lock().unwrap();
                *current = Some(LoadedModel {
                    name: model_name.to_string(),
                    client,
                });
                debug!(
                    "<<< LOAD COMPLETE (sync): Model '{}' - now in memory and ready",
                    model_name
                );
                info!("Model '{}' loaded successfully", model_name);
                Ok(())
            }
            Err(e) => {
                let error = format!("Failed to load model: {}", e);
                error!("{}", error);
                Err(error)
            }
        }
    }
}
