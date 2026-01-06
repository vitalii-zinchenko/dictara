# Implementation Plan: Local Model Transcription for Dictara

## Summary

Add local Whisper model transcription as a third provider option alongside OpenAI and Azure OpenAI. Users can download models within the app and transcribe audio 100% offline.

**Key Architecture Decision**: Local provider follows the same pattern as existing providers - when selected, the section expands to show model management UI instead of API key inputs.

---

## Phase 1: Rust Backend - Core Infrastructure

### 1.1 Add Dependencies

**File**: [src-tauri/Cargo.toml](src-tauri/Cargo.toml)

```toml
# Local transcription - whisper.cpp bindings with Metal acceleration
whisper-rs = { version = "0.13", features = ["metal"] }

# For async model downloads with progress
futures-util = "0.3"
```

> Note: `whisper-rs` wraps whisper.cpp and automatically uses Metal on macOS for GPU acceleration.

### 1.2 Create Models Module

**New files to create**:

| File | Purpose |
|------|---------|
| `src-tauri/src/models/mod.rs` | Module exports |
| `src-tauri/src/models/catalog.rs` | Static model catalog (hardcoded list) |
| `src-tauri/src/models/status.rs` | Runtime model status |
| `src-tauri/src/models/manager.rs` | Model download, storage, status tracking |
| `src-tauri/src/models/local_client.rs` | Whisper inference wrapper |

### 1.3 Data Structures

#### ModelCatalogEntry (Static, Hardcoded)

**File**: `src-tauri/src/models/catalog.rs`

```rust
/// Static information about a model available for download.
/// This is hardcoded and never changes at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogEntry {
    pub name: String,              // "whisper-small" (unique ID, no spaces)
    pub display_name: String,      // "Whisper Small"
    pub description: String,       // "Fast, good for most use cases"
    pub filename: String,          // "ggml-small.bin"
    pub url: String,               // Hugging Face URL
    pub estimated_size_bytes: u64, // Approximate size on disk
    pub estimated_ram_mb: u64,     // Approximate RAM when loaded
}

/// Hardcoded catalog of available models
pub fn get_model_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        ModelCatalogEntry {
            name: "whisper-small".into(),
            display_name: "Whisper Small".into(),
            description: "Fast, good for most use cases. Recommended for 8GB RAM.".into(),
            filename: "ggml-small.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
            estimated_size_bytes: 466_000_000,  // ~466 MB
            estimated_ram_mb: 800,
        },
        ModelCatalogEntry {
            name: "whisper-medium".into(),
            display_name: "Whisper Medium".into(),
            description: "Better accuracy, requires more RAM. Recommended for 16GB RAM.".into(),
            filename: "ggml-medium.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
            estimated_size_bytes: 1_500_000_000,  // ~1.5 GB
            estimated_ram_mb: 2200,
        },
        ModelCatalogEntry {
            name: "whisper-large-v3-turbo".into(),
            display_name: "Whisper Large v3 Turbo".into(),
            description: "Fast large model variant. Recommended for 16GB RAM.".into(),
            filename: "ggml-large-v3-turbo.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".into(),
            estimated_size_bytes: 1_600_000_000,  // ~1.6 GB
            estimated_ram_mb: 2500,
        },
        ModelCatalogEntry {
            name: "whisper-large-v3".into(),
            display_name: "Whisper Large v3".into(),
            description: "Best accuracy, requires significant RAM. Recommended for 16GB+ RAM.".into(),
            filename: "ggml-large-v3.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".into(),
            estimated_size_bytes: 3_000_000_000,  // ~3 GB
            estimated_ram_mb: 4000,
        },
    ]
}
```

#### ModelStatus (Runtime, Computed)

**File**: `src-tauri/src/models/status.rs`

```rust
/// Runtime status of a model - computed, not stored.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub is_downloaded: bool,    // Does file exist on disk?
    pub is_downloading: bool,   // In-memory flag (reset on app restart)
    pub is_loaded: bool,        // Is model currently loaded in memory?
    pub is_loading: bool,       // Is model currently being loaded?
    pub downloaded_bytes: u64,  // Size of .partial file (for resume)
}
```

**Where each field comes from:**

| Field | Storage | How to compute |
|-------|---------|----------------|
| `is_downloaded` | Computed | `models/{filename}` exists? |
| `is_downloading` | In-memory | `HashMap<String, bool>` in ModelManager |
| `is_loaded` | In-memory | Check ModelLoader.current_model |
| `is_loading` | In-memory | `HashMap<String, bool>` in ModelLoader |
| `downloaded_bytes` | Computed | File size of `models/{filename}.partial` |

#### ModelInfo (Merged view for Frontend)

**File**: `src-tauri/src/models/mod.rs`

```rust
/// Combined view sent to frontend (catalog + status merged).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    // From catalog
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub estimated_size_bytes: u64,
    pub estimated_ram_mb: u64,

    // From status
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub is_loaded: bool,
    pub is_loading: bool,
    pub downloaded_bytes: u64,
}

impl ModelInfo {
    pub fn from_catalog_and_status(catalog: &ModelCatalogEntry, status: &ModelStatus) -> Self {
        Self {
            name: catalog.name.clone(),
            display_name: catalog.display_name.clone(),
            description: catalog.description.clone(),
            estimated_size_bytes: catalog.estimated_size_bytes,
            estimated_ram_mb: catalog.estimated_ram_mb,
            is_downloaded: status.is_downloaded,
            is_downloading: status.is_downloading,
            is_loaded: status.is_loaded,
            is_loading: status.is_loading,
            downloaded_bytes: status.downloaded_bytes,
        }
    }
}
```

#### DownloadProgress (Event payload)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub model_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}
```

### 1.4 Model Manager

**File**: `src-tauri/src/models/manager.rs`

Key responsibilities:
- Get catalog entries
- Compute model status by checking filesystem
- Track in-memory downloading state
- Download with progress events via Tauri
- Resume interrupted downloads (check .partial file size)
- Model deletion

**Model Storage**: `~/Library/Application Support/app.dictara/models/`

```rust
pub struct ModelManager {
    models_dir: PathBuf,
    downloading: Arc<Mutex<HashMap<String, bool>>>,  // In-memory download state
    cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl ModelManager {
    /// Get all models with their current status
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        get_model_catalog()
            .iter()
            .map(|entry| {
                let status = self.compute_status(entry);
                ModelInfo::from_catalog_and_status(entry, &status)
            })
            .collect()
    }

    /// Compute runtime status for a model
    fn compute_status(&self, entry: &ModelCatalogEntry) -> ModelStatus {
        let model_path = self.models_dir.join(&entry.filename);
        let partial_path = self.models_dir.join(format!("{}.partial", entry.filename));

        ModelStatus {
            is_downloaded: model_path.exists(),
            is_downloading: self.downloading.lock().unwrap()
                .get(&entry.name).copied().unwrap_or(false),
            downloaded_bytes: partial_path.metadata()
                .map(|m| m.len()).unwrap_or(0),
        }
    }
}
```

### 1.5 Local Client (Whisper Inference)

**File**: `src-tauri/src/models/local_client.rs`

```rust
pub struct LocalClient {
    context: WhisperContext,  // Always loaded when LocalClient exists
}

impl LocalClient {
    /// Load model into memory. This is called eagerly, not lazily.
    /// Load time: ~1-2s (small) to ~6-10s (large v3)
    pub fn new(model_path: &Path) -> Result<Self, TranscriptionError> {
        let ctx = WhisperContext::new(&model_path.to_string_lossy())
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;
        Ok(Self { context: ctx })
    }

    pub fn transcribe_file(&self, audio_path: &Path) -> Result<String, TranscriptionError> {
        // Load audio as f32 samples at 16kHz
        let samples = self.load_audio(audio_path)?;

        // Transcribe using whisper-rs
        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let mut state = self.context.create_state()?;
        state.full(params, &samples)?;

        // Extract text from segments
        self.extract_text(&state)
    }
}
```

### 1.6 Model Loader (Manages Loaded Model State)

**File**: `src-tauri/src/models/loader.rs`

Manages the currently loaded model as app-level state.

```rust
pub struct ModelLoader {
    current_model: Arc<Mutex<Option<LoadedModel>>>,
    models_dir: PathBuf,
}

struct LoadedModel {
    name: String,           // e.g., "whisper-small"
    client: LocalClient,
}

impl ModelLoader {
    /// Load a model (unloads previous if any)
    pub async fn load_model(&self, model_name: &str, app: &AppHandle) -> Result<(), String> {
        // Emit loading-started event to frontend
        app.emit("model-loading-started", model_name)?;

        // Unload current model if any
        self.unload_model();

        // Look up model in catalog
        let entry = get_model_catalog()
            .into_iter()
            .find(|e| e.name == model_name)
            .ok_or("Model not found in catalog")?;

        let model_path = self.models_dir.join(&entry.filename);

        // Load in background thread (blocking operation)
        let client = tokio::task::spawn_blocking(move || {
            LocalClient::new(&model_path)
        }).await??;

        // Store loaded model
        let mut current = self.current_model.lock().unwrap();
        *current = Some(LoadedModel {
            name: model_name.to_string(),
            client,
        });

        // Emit loading-complete event
        app.emit("model-loading-complete", model_name)?;
        Ok(())
    }

    /// Unload current model (frees memory)
    pub fn unload_model(&self) {
        let mut current = self.current_model.lock().unwrap();
        *current = None;
    }

    /// Get reference to loaded client for transcription
    pub fn get_client(&self) -> Option<&LocalClient> {
        self.current_model.lock().unwrap()
            .as_ref()
            .map(|m| &m.client)
    }

    /// Check if a specific model is loaded
    pub fn is_model_loaded(&self, model_name: &str) -> bool {
        self.current_model.lock().unwrap()
            .as_ref()
            .map(|m| m.name == model_name)
            .unwrap_or(false)
    }

    /// Get name of currently loaded model
    pub fn get_loaded_model_name(&self) -> Option<String> {
        self.current_model.lock().unwrap()
            .as_ref()
            .map(|m| m.name.clone())
    }
}
```

**Eager Loading Behavior:**

| Event | Action |
|-------|--------|
| App starts | If provider=Local + model selected + file exists → load model |
| User selects different model | Unload old → load new |
| User switches TO Local provider | Load selected model (if exists) |
| User switches AWAY from Local | Unload model (free memory) |
| User deletes currently loaded model | Unload model |

**Load Times (approximate):**
- Small: ~1-2 seconds
- Medium: ~3-5 seconds
- Large v3 Turbo: ~3-5 seconds
- Large v3: ~6-10 seconds

**UI Feedback:**
- Emit `model-loading-started` event → frontend shows "Loading model..."
- Emit `model-loading-complete` event → frontend enables transcription
- Disable transcription button while loading

---

## Phase 2: Integrate with Existing Architecture

### 2.1 Update Provider Enum

**File**: [src-tauri/src/config.rs](src-tauri/src/config.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
pub enum Provider {
    #[serde(rename = "open_ai")]
    OpenAI,
    #[serde(rename = "azure_open_ai")]
    AzureOpenAI,
    #[serde(rename = "local")]
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelConfig {
    pub selected_model: Option<String>,  // e.g., "whisper-small" (references catalog name)
}
```

### 2.2 Update Transcriber

**File**: [src-tauri/src/clients/transcriber.rs](src-tauri/src/clients/transcriber.rs)

```rust
Provider::Local => {
    let config: LocalModelConfig = load_local_model_config(store)?;
    let model_name = config.selected_model
        .ok_or(TranscriptionError::NoModelSelected)?;

    // Look up filename from catalog
    let catalog_entry = get_model_catalog()
        .into_iter()
        .find(|e| e.name == model_name)
        .ok_or(TranscriptionError::ModelNotFound(model_name.clone()))?;

    let model_path = get_models_dir()?.join(&catalog_entry.filename);

    if !model_path.exists() {
        return Err(TranscriptionError::ModelNotDownloaded(model_name));
    }

    Ok(Box::new(LocalClient::new(model_path)))
}
```

### 2.3 Update Error Types

**File**: [src-tauri/src/clients/error.rs](src-tauri/src/clients/error.rs)

```rust
pub enum TranscriptionError {
    // ... existing errors
    NoModelSelected,
    ModelNotFound(String),
    ModelNotDownloaded(String),
    ModelLoadFailed(String),
    LocalTranscriptionFailed(String),
}
```

---

## Phase 3: Tauri Commands

**File**: [src-tauri/src/tauri_commands.rs](src-tauri/src/tauri_commands.rs)

```rust
/// Get list of all models with their current status
#[tauri::command]
#[specta::specta]
pub async fn get_available_models(
    model_manager: State<'_, Arc<ModelManager>>,
) -> Result<Vec<ModelInfo>, String>

/// Start model download (emits progress events)
#[tauri::command]
#[specta::specta]
pub async fn download_model(
    model_manager: State<'_, Arc<ModelManager>>,
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String>

/// Cancel ongoing download
#[tauri::command]
#[specta::specta]
pub async fn cancel_model_download(
    model_manager: State<'_, Arc<ModelManager>>,
    model_name: String,
) -> Result<(), String>

/// Delete downloaded model
#[tauri::command]
#[specta::specta]
pub async fn delete_model(
    model_manager: State<'_, Arc<ModelManager>>,
    model_name: String,
) -> Result<(), String>

/// Save local model config (which model to use)
#[tauri::command]
#[specta::specta]
pub fn save_local_model_config(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String>

/// Load local model config
#[tauri::command]
#[specta::specta]
pub fn load_local_model_config(
    app: tauri::AppHandle,
) -> Result<Option<LocalModelConfig>, String>

/// Delete local model config
#[tauri::command]
#[specta::specta]
pub fn delete_local_model_config(
    app: tauri::AppHandle,
) -> Result<(), String>
```

/// Load a model into memory (for transcription)
#[tauri::command]
#[specta::specta]
pub async fn load_model(
    model_loader: State<'_, Arc<ModelLoader>>,
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String>

/// Unload currently loaded model (free memory)
#[tauri::command]
#[specta::specta]
pub async fn unload_model(
    model_loader: State<'_, Arc<ModelLoader>>,
) -> Result<(), String>

/// Get current model loading status
#[tauri::command]
#[specta::specta]
pub fn get_loaded_model(
    model_loader: State<'_, Arc<ModelLoader>>,
) -> Option<String>
```

**Tauri Events:**

| Event | Payload | When |
|-------|---------|------|
| `model-download-progress` | `DownloadProgress` | During download |
| `model-download-complete` | `model_name: String` | Download finished |
| `model-download-error` | `{ model_name, error }` | Download failed |
| `model-loading-started` | `model_name: String` | Model loading into memory |
| `model-loading-complete` | `model_name: String` | Model ready for transcription |
| `model-loading-error` | `{ model_name, error }` | Model failed to load |

---

## Phase 4: Frontend UI

### 4.1 Add Local Provider Component

**New file**: `src/components/preferences/api-keys/LocalProvider.tsx`

```tsx
export function LocalProvider({ isExpanded, isActive, onToggleExpand, onToggleActive }) {
  const { data: models } = useAvailableModels();
  const { data: config } = useLocalModelConfig();

  return (
    <ProviderCard
      name="Local (Offline)"
      description="Transcribe locally using Whisper models. No API key required."
      isExpanded={isExpanded}
      isActive={isActive}
      onToggleExpand={() => onToggleExpand('local')}
      onToggleActive={() => onToggleActive('local')}
    >
      {isExpanded && (
        <ModelSelector
          models={models}
          selectedModel={config?.selectedModel}
          onSelectModel={handleSelectModel}
        />
      )}
    </ProviderCard>
  );
}
```

### 4.2 Model Selector Component

**New file**: `src/components/preferences/api-keys/ModelSelector.tsx`

Shows list of available models with:
- Model display name, description
- Estimated disk size and RAM requirement
- Download/Delete button based on status
- Progress bar when downloading
- Radio to select downloaded model
- Visual indicator for currently selected model

### 4.3 Model Download Progress Component

**New file**: `src/components/preferences/api-keys/ModelDownloadProgress.tsx`

Shows:
- Progress bar with percentage
- Downloaded / Total (e.g., "234 MB / 466 MB")
- Cancel button

### 4.4 Update Provider Types

**File**: [src/components/preferences/api-keys/types.ts](src/components/preferences/api-keys/types.ts)

```typescript
export type Provider = 'open_ai' | 'azure_open_ai' | 'local' | null;
```

### 4.5 Update API Keys Page

**File**: [src/components/preferences/api-keys/index.tsx](src/components/preferences/api-keys/index.tsx)

Add LocalProvider alongside existing providers.

### 4.6 Add Hooks

**New file**: `src/hooks/useModels.ts`

```typescript
export function useAvailableModels() { ... }
export function useDownloadModel() { ... }
export function useCancelModelDownload() { ... }
export function useDeleteModel() { ... }
export function useLocalModelConfig() { ... }
export function useSaveLocalModelConfig() { ... }
export function useDeleteLocalModelConfig() { ... }
```

---

## Phase 5: Initialization

**File**: [src-tauri/src/lib.rs](src-tauri/src/lib.rs)

```rust
let model_manager = Arc::new(ModelManager::new(&app)?);
app.manage(model_manager);
```

---

## File Summary

### New Files (10)

| Path | Purpose |
|------|---------|
| `src-tauri/src/models/mod.rs` | Module exports, ModelInfo |
| `src-tauri/src/models/catalog.rs` | Static model catalog |
| `src-tauri/src/models/status.rs` | Runtime status struct |
| `src-tauri/src/models/manager.rs` | Model download/delete management |
| `src-tauri/src/models/loader.rs` | Model loading/unloading (memory) |
| `src-tauri/src/models/local_client.rs` | Whisper inference |
| `src/components/preferences/api-keys/LocalProvider.tsx` | UI component |
| `src/components/preferences/api-keys/ModelSelector.tsx` | Model list UI |
| `src/components/preferences/api-keys/ModelDownloadProgress.tsx` | Progress UI |
| `src/hooks/useModels.ts` | React Query hooks |

### Modified Files (7)

| Path | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add whisper-rs, futures-util |
| `src-tauri/src/lib.rs` | Add models module, register commands, init ModelManager |
| `src-tauri/src/config.rs` | Add Provider::Local, LocalModelConfig |
| `src-tauri/src/clients/transcriber.rs` | Handle local provider |
| `src-tauri/src/clients/error.rs` | Add local model errors |
| `src-tauri/src/tauri_commands.rs` | Add model management commands |
| `src/components/preferences/api-keys/index.tsx` | Add LocalProvider |

---

## Implementation Order

1. **Backend Core** (can be tested independently)
   - Add Cargo dependencies
   - Create models module with catalog + status structs
   - Implement ModelManager
   - Implement LocalClient with whisper-rs

2. **Integration**
   - Update Provider enum and config
   - Update Transcriber to handle local
   - Add Tauri commands

3. **Frontend**
   - Add types and hooks
   - Create LocalProvider component
   - Create ModelSelector component
   - Integrate into API Keys page

4. **Testing**
   - Test model download/delete
   - Test local transcription
   - Test provider switching

---

## Data Flow Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                        STATIC (Hardcoded)                       │
│  ModelCatalogEntry[]                                            │
│  - name, display_name, description, filename, url               │
│  - estimated_size_bytes, estimated_ram_mb                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     RUNTIME (Computed/In-Memory)                │
│  ModelStatus                                                    │
│  - is_downloaded: check if models/{filename} exists             │
│  - is_downloading: in-memory HashMap                            │
│  - downloaded_bytes: size of models/{filename}.partial          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MERGED (Sent to Frontend)                   │
│  ModelInfo = ModelCatalogEntry + ModelStatus                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Notes

- **Model Storage**: `~/Library/Application Support/app.dictara/models/`
- **Partial Files**: `{filename}.partial` - used for resume support
- **Memory**: Models are lazy-loaded on first transcription, not on app start
- **GPU**: Metal acceleration is automatic on macOS via whisper-rs
- **Existing VAD**: Dictara already has Silero VAD, which will work with local transcription
- **Backward Compatible**: Existing OpenAI/Azure configs remain unchanged
