# Keyboard Shortcuts Configuration - Full Implementation Plan

## Summary

This plan evolves the existing simple trigger key feature into a comprehensive 3-shortcut system with key combinations, runtime hot-swapping, and interactive key capture UI.

**Architecture Approach:** Uses a simple **mpsc channel-based** hot-swap mechanism instead of `Arc<RwLock<>>` for better performance and simpler code. KeyListener caches shortcuts locally and checks for updates via non-blocking `try_recv()` on each event.

**Current State (Already Implemented):**
- Simple `RecordingTrigger` enum (Fn/Control/Option/Command) in [config.rs:19-40](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L19-L40)
- Type-safe `ConfigStore` pattern with get/set/delete methods in [config.rs:160-199](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L160-L199)
- Single trigger passed to keyboard listener in [keyboard_listener.rs:20](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs#L20)
- Space hardcoded as LOCK_MODIFIER for hands-free mode (line 9)
- TriggerKeyStep onboarding component with app restart flow
- Hotkeys preferences page with restart button
- Changes require app restart

**Target State:**
- Three separate configurable shortcuts with key combinations (1-3 keys each)
- Runtime hot-swap (no restart required)
- Interactive key capture UI
- Backward compatible migration from current implementation

---

## User Requirements

### Three Configurable Shortcuts

1. **Push to Record** (default: Fn)
   - Hold to record, release to transcribe
   - Current behavior: single trigger key

2. **Hands-free Start** (default: Fn + Space)
   - Toggle recording on (locked mode)
   - Current: hardcoded as trigger + Space

3. **Hands-free Stop** (default: Fn)
   - Stop hands-free recording
   - Current: same as push-to-record trigger

### Key Features

- **Key Combinations**: Up to 3 keys per shortcut (e.g., Shift+Cmd+R, Fn+Space)
- **Runtime Hot-Swap**: Changes take effect immediately without app restart
- **Key Capture UI**: Press actual keys to record combinations (special handling for Fn key)
- **Migration**: Seamlessly upgrade existing users from simple trigger to 3-shortcut system

---

## Architecture Design

### Backend Data Structures

Add to [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs):

```rust
/// A single key in a shortcut combination
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutKey {
    pub keycode: u32,           // macOS keycode (e.g., 63 for Fn)
    pub label: String,          // Display name (e.g., "Fn", "Space")
}

/// A keyboard shortcut (1-3 keys)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub keys: Vec<ShortcutKey>,
}

impl Shortcut {
    /// Check if this shortcut matches currently pressed keys
    pub fn matches(&self, pressed_keys: &HashSet<u32>) -> bool {
        // Exact match: same count AND all keys present
        self.keys.len() == pressed_keys.len()
            && self.keys.iter().all(|k| pressed_keys.contains(&k.keycode))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() || self.keys.len() > 3 {
            return Err("Shortcut must have 1-3 keys".to_string());
        }
        // Check for duplicates
        let mut seen = HashSet::new();
        for key in &self.keys {
            if !seen.insert(key.keycode) {
                return Err(format!("Duplicate key: {}", key.label));
            }
        }
        Ok(())
    }
}

/// Complete shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutsConfig {
    pub push_to_record: Shortcut,
    pub hands_free_start: Shortcut,
    pub hands_free_stop: Shortcut,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            push_to_record: Shortcut {
                keys: vec![ShortcutKey { keycode: 63, label: "Fn".into() }],
            },
            hands_free_start: Shortcut {
                keys: vec![
                    ShortcutKey { keycode: 63, label: "Fn".into() },
                    ShortcutKey { keycode: 49, label: "Space".into() },
                ],
            },
            hands_free_stop: Shortcut {
                keys: vec![ShortcutKey { keycode: 63, label: "Fn".into() }],
            },
        }
    }
}

impl ConfigKey<ShortcutsConfig> {
    pub const SHORTCUTS: Self = Self::new("shortcutsConfig");
}
```

### Migration Logic

Add to [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs):

```rust
/// Migrate from RecordingTrigger to ShortcutsConfig (run once on startup)
pub fn migrate_trigger_to_shortcuts(store: &impl ConfigStore) -> Result<(), String> {
    // Skip if already migrated
    if store.get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS).is_some() {
        return Ok(());
    }

    // Load old trigger config
    let app_config = store.get(&ConfigKey::APP).unwrap_or_default();
    let trigger_key = match app_config.recording_trigger {
        RecordingTrigger::Fn => ShortcutKey { keycode: 63, label: "Fn".into() },
        RecordingTrigger::Control => ShortcutKey { keycode: 59, label: "Control".into() },
        RecordingTrigger::Option => ShortcutKey { keycode: 58, label: "Option".into() },
        RecordingTrigger::Command => ShortcutKey { keycode: 55, label: "Command".into() },
    };

    // Create new shortcuts config preserving user's trigger choice
    let shortcuts = ShortcutsConfig {
        push_to_record: Shortcut { keys: vec![trigger_key] },
        hands_free_start: Shortcut {
            keys: vec![trigger_key, ShortcutKey { keycode: 49, label: "Space".into() }],
        },
        hands_free_stop: Shortcut { keys: vec![trigger_key] },
    };

    store.set(&ConfigKey::<ShortcutsConfig>::SHORTCUTS, shortcuts)?;
    Ok(())
}
```

### Channel-Based Hot-Swap Architecture

**No separate state component needed!** KeyListener caches config locally and receives updates via mpsc channel.

**Why this is simpler:**
- No `Arc<RwLock<>>` overhead
- No separate ShortcutsState file/module
- Faster: local variable read vs RwLock read
- More idiomatic Rust for thread communication

### Updated Keyboard Listener

Modify [keyboard_listener.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs):

**Key Changes:**
1. Replace `recording_trigger: Key` parameter with `initial_config: ShortcutsConfig`
2. Create mpsc channel for config updates: `(config_tx, config_rx)`
3. **Cache shortcuts locally** in the spawned thread (no locks!)
4. Check `config_rx.try_recv()` on each event to hot-swap config
5. Track pressed keys in a `HashSet<u32>` for combination matching
6. Match shortcuts using `shortcut.matches(&pressed_keys)`
7. Remove hardcoded LOCK_MODIFIER constant
8. Store `config_tx` in KeyListener struct for `update_shortcuts()` method

```rust
use crate::config::ShortcutsConfig;
use std::collections::HashSet;
use tokio::sync::mpsc;

pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
    config_tx: mpsc::Sender<ShortcutsConfig>, // Send config updates to thread
}

impl KeyListener {
    pub fn start(
        command_tx: mpsc::Sender<RecordingCommand>,
        state_manager: Arc<RecordingStateManager>,
        initial_config: ShortcutsConfig,
    ) -> Self {
        let (config_tx, mut config_rx) = mpsc::channel(10);

        let thread_handle = thread::spawn(move || {
            // CACHE shortcuts locally - NO LOCKS!
            let mut shortcuts = initial_config;
            let mut pressed_keys: HashSet<u32> = HashSet::new();

            if let Err(err) = grab(move |event| {
                // Check for config updates (non-blocking, ~5ns overhead)
                if let Ok(new_config) = config_rx.try_recv() {
                    shortcuts = new_config; // Hot-swap!
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let keycode = key.to_macos_keycode();
                        pressed_keys.insert(keycode);

                        // Check shortcuts (reads LOCAL variable - fast!)
                        if shortcuts.push_to_record.matches(&pressed_keys) {
                            let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                            return Some(event);
                        }

                        if shortcuts.hands_free_start.matches(&pressed_keys) {
                            let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                            let _ = command_tx.blocking_send(RecordingCommand::LockRecording);
                            // Swallow Space if it's in the combo
                            if shortcuts.hands_free_start.keys.iter().any(|k| k.keycode == 49) {
                                return None;
                            }
                        }

                        Some(event)
                    }
                    EventType::KeyRelease(key) => {
                        let keycode = key.to_macos_keycode();

                        // Check push-to-record BEFORE removing key
                        let was_push_to_record = shortcuts.push_to_record.matches(&pressed_keys);

                        pressed_keys.remove(&keycode);

                        // Release stops recording (unless locked)
                        if was_push_to_record && !state_manager.is_recording_locked() {
                            let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        }

                        // Check hands-free stop
                        if shortcuts.hands_free_stop.matches(&pressed_keys)
                            && state_manager.is_recording_locked() {
                            let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        }

                        Some(event)
                    }
                    _ => Some(event),
                }
            }) {
                error!("Keyboard grab failed: {}", err);
            }
        });

        Self {
            _thread_handle: Some(thread_handle),
            config_tx,
        }
    }

    /// Update shortcuts at runtime (no restart needed!)
    pub fn update_shortcuts(&self, new_config: ShortcutsConfig) -> Result<(), String> {
        self.config_tx.blocking_send(new_config)
            .map_err(|_| "KeyListener thread is not running".to_string())
    }

    /// Check if any shortcut uses Fn key (for globe key fix)
    pub fn uses_fn_key(config: &ShortcutsConfig) -> bool {
        let fn_code = 63u32;
        config.push_to_record.keys.iter().any(|k| k.keycode == fn_code)
            || config.hands_free_start.keys.iter().any(|k| k.keycode == fn_code)
            || config.hands_free_stop.keys.iter().any(|k| k.keycode == fn_code)
    }
}
```

**Note:** Need to add `to_macos_keycode()` method to `dictara_keyboard::Key` enum in `crates/keyboard/src/key.rs`.

### Backend Commands

Create new file: `src-tauri/src/commands/preferences/shortcuts.rs`

```rust
use crate::config::{ConfigKey, ConfigStore, ShortcutsConfig};
use crate::keyboard_listener::KeyListener;
use crate::shortcuts::events::KeyCaptureEvent;
use tauri::{AppHandle, State};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Global state to control key capture listener
pub struct KeyCaptureState {
    is_capturing: Arc<Mutex<bool>>,
}

impl KeyCaptureState {
    pub fn new() -> Self {
        Self {
            is_capturing: Arc::new(Mutex::new(false)),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn load_shortcuts_config(
    config_store: State<config::Config>,
) -> Result<ShortcutsConfig, String> {
    Ok(config_store.get(&ConfigKey::SHORTCUTS).unwrap_or_default())
}

#[tauri::command]
#[specta::specta]
pub fn save_shortcuts_config(
    config_store: State<config::Config>,
    key_listener: State<KeyListener>,
    config: ShortcutsConfig,
) -> Result<(), String> {
    // Validate all shortcuts
    config.push_to_record.validate()?;
    config.hands_free_start.validate()?;
    config.hands_free_stop.validate()?;

    // Load old config for Fn key change detection
    let old_config = config_store.get(&ConfigKey::SHORTCUTS).unwrap_or_default();
    let old_uses_fn = KeyListener::uses_fn_key(&old_config);
    let new_uses_fn = KeyListener::uses_fn_key(&config);

    // Save to persistent storage
    config_store.set(&ConfigKey::SHORTCUTS, config.clone())?;

    // Hot-swap runtime config via channel (NO RESTART NEEDED!)
    key_listener.update_shortcuts(config)?;

    // Update globe key fix if Fn usage changed
    if !old_uses_fn && new_uses_fn {
        crate::globe_key::fix_globe_key_if_needed();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn reset_shortcuts_config(
    config_store: State<config::Config>,
    key_listener: State<KeyListener>,
) -> Result<ShortcutsConfig, String> {
    let defaults = ShortcutsConfig::default();
    config_store.set(&ConfigKey::SHORTCUTS, defaults.clone())?;
    key_listener.update_shortcuts(defaults.clone())?;
    Ok(defaults)
}

#[tauri::command]
#[specta::specta]
pub async fn start_key_capture(
    app_handle: AppHandle,
    capture_state: State<'_, KeyCaptureState>,
) -> Result<(), String> {
    let mut is_capturing = capture_state.is_capturing.lock().await;

    if *is_capturing {
        return Err("Key capture already in progress".to_string());
    }

    *is_capturing = true;
    let is_capturing_clone = capture_state.is_capturing.clone();

    // Spawn keyboard listener in separate thread
    std::thread::spawn(move || {
        use dictara_keyboard::{grab, EventType};

        let _ = grab(move |event| {
            // Check if still capturing
            let should_continue = *is_capturing_clone.blocking_lock();
            if !should_continue {
                return None; // Stop grabbing
            }

            match event.event_type {
                EventType::KeyPress(key) => {
                    let keycode = key.to_macos_keycode();
                    let label = key.to_label();

                    let _ = KeyCaptureEvent::KeyDown { keycode, label }
                        .emit(&app_handle);
                }
                EventType::KeyRelease(key) => {
                    let keycode = key.to_macos_keycode();
                    let label = key.to_label();

                    let _ = KeyCaptureEvent::KeyUp { keycode, label }
                        .emit(&app_handle);
                }
                _ => {}
            }

            // Don't swallow keys during capture (user can see what they're typing)
            Some(event)
        });
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_key_capture(
    capture_state: State<'_, KeyCaptureState>,
) -> Result<(), String> {
    let mut is_capturing = capture_state.is_capturing.lock().await;
    *is_capturing = false;
    Ok(())
}
```

Register in:
- `src-tauri/src/commands/mod.rs`: add `pub mod shortcuts;`
- `src-tauri/src/commands/registry.rs`: add all 5 commands to the list
- `src-tauri/src/setup.rs`: add `app.manage(KeyCaptureState::new());`

### App Setup Integration

Modify [setup.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/setup.rs):

```rust
// After line 158 (state_manager creation), add:

// Load or migrate shortcuts config
migrate_trigger_to_shortcuts(&config_store)?;
let shortcuts_config = config_store
    .get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS)
    .unwrap_or_default();

// Replace lines 204-206 (KeyListener initialization):
if has_accessibility {
    let listener = KeyListener::start(
        command_tx,
        state_manager.clone(),
        shortcuts_config.clone(), // Pass initial config (cached in thread!)
    );

    // Manage KeyListener in Tauri state for hot-swapping
    app.manage(listener);
}

// Update lines 216-220 (globe key fix):
if KeyListener::uses_fn_key(&shortcuts_config) {
    globe_key::fix_globe_key_if_needed();
}
```

**No need to modify `src-tauri/src/lib.rs`** - no separate shortcuts_state module!

---

## Frontend Implementation

### Key Capture Architecture: Backend-Streamed Events

**Critical Design Decision:** JavaScript keycode mismatches with macOS keycodes used by Rust!
- JavaScript `event.keyCode`: Virtual key codes (e.g., Space = 32, A = 65)
- macOS keycodes (rdev): Hardware scan codes (e.g., Space = 49, A = 0)
- **Solution:** Backend captures keys and streams events to frontend

**Architecture:**
1. Frontend clicks "Capture Keys" → calls `start_key_capture()` command
2. Backend spawns temporary keyboard listener using `dictara_keyboard`
3. Backend emits `KeyCaptureEvent` (KeyDown/KeyUp) with correct macOS keycodes
4. Frontend listens to event stream, updates UI in real-time
5. User clicks "Done" → calls `stop_key_capture()` command
6. Frontend saves captured keys with matching keycodes

**Benefits:**
- ✅ No keycode mismatch (Rust captures same codes it uses for listening)
- ✅ Can capture Fn key (backend has OS-level access)
- ✅ Real-time visual feedback as keys are pressed
- ✅ Consistent with existing tauri-specta event pattern
- ✅ Order preserved in array

### Key Capture Events

Add to `src-tauri/src/shortcuts/events.rs` (new file):

```rust
use serde::{Deserialize, Serialize};

/// Key capture event - streamed to frontend during shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyCaptureEvent {
    /// Key was pressed
    #[serde(rename = "keyDown")]
    KeyDown {
        keycode: u32,
        label: String,
    },
    /// Key was released
    #[serde(rename = "keyUp")]
    KeyUp {
        keycode: u32,
        label: String,
    },
}
```

Register in `src-tauri/src/lib.rs`:
```rust
mod shortcuts {
    pub mod events;
}
```

Add to tauri-specta builder in `src-tauri/src/lib.rs`:
```rust
.event::<shortcuts::events::KeyCaptureEvent>()
```

### Frontend: Key Capture Hook

Create `src/hooks/useKeyCapture.ts`:

```tsx
import { useEffect, useState, useCallback } from 'react'
import { events, commands } from '@/bindings'

export interface CapturedKey {
  keycode: number
  label: string
}

export function useKeyCapture() {
  const [isCapturing, setIsCapturing] = useState(false)
  const [pressedKeys, setPressedKeys] = useState<CapturedKey[]>([])

  const startCapture = useCallback(async () => {
    setPressedKeys([])
    setIsCapturing(true)
    await commands.startKeyCapture()
  }, [])

  const stopCapture = useCallback(async () => {
    setIsCapturing(false)
    await commands.stopKeyCapture()
  }, [])

  const clearKeys = useCallback(() => {
    setPressedKeys([])
  }, [])

  // Listen to key events from backend
  useEffect(() => {
    if (!isCapturing) return

    const setupListener = async () => {
      const unlisten = await events.keyCaptureEvent.listen((event) => {
        const payload = event.payload

        if (payload.type === 'keyDown') {
          setPressedKeys((prev) => {
            // Avoid duplicates
            if (prev.some(k => k.keycode === payload.keycode)) {
              return prev
            }
            // Max 3 keys
            if (prev.length >= 3) {
              return prev
            }
            return [...prev, { keycode: payload.keycode, label: payload.label }]
          })
        } else if (payload.type === 'keyUp') {
          setPressedKeys((prev) =>
            prev.filter(k => k.keycode !== payload.keycode)
          )
        }
      })

      return unlisten
    }

    let cleanup: (() => void) | undefined
    setupListener().then((cleanupFn) => {
      cleanup = cleanupFn
    })

    return () => {
      if (cleanup) cleanup()
    }
  }, [isCapturing])

  return {
    isCapturing,
    pressedKeys,
    startCapture,
    stopCapture,
    clearKeys,
  }
}
```

### Frontend: Key Capture Component

Create `src/components/shortcuts/KeyCaptureInput.tsx`:

**Features:**
- Uses `useKeyCapture` hook to receive backend events
- Displays current keys as badges with remove buttons
- Captures ALL keys including Fn (via backend!)
- Max 3 keys enforcement
- Real-time visual feedback

```tsx
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useKeyCapture } from '@/hooks/useKeyCapture'
import { X } from 'lucide-react'

interface KeyCaptureInputProps {
  value: CapturedKey[]
  onChange: (keys: CapturedKey[]) => void
  label: string
}

export function KeyCaptureInput({ value, onChange, label }: KeyCaptureInputProps) {
  const { isCapturing, pressedKeys, startCapture, stopCapture } = useKeyCapture()

  const handleStartCapture = () => {
    startCapture()
  }

  const handleDone = () => {
    stopCapture()
    onChange(pressedKeys)
  }

  const handleCancel = () => {
    stopCapture()
    // Don't update value
  }

  const handleRemoveKey = (keycode: number) => {
    onChange(value.filter(k => k.keycode !== keycode))
  }

  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">{label}</label>

      <div className="flex items-center gap-2 flex-wrap min-h-[40px] border rounded-md p-2">
        {(isCapturing ? pressedKeys : value).map((key) => (
          <Badge key={key.keycode} variant="secondary" className="gap-1">
            {key.label}
            {!isCapturing && (
              <X
                className="h-3 w-3 cursor-pointer"
                onClick={() => handleRemoveKey(key.keycode)}
              />
            )}
          </Badge>
        ))}

        {!isCapturing && value.length === 0 && (
          <span className="text-sm text-muted-foreground">No keys assigned</span>
        )}
      </div>

      {isCapturing ? (
        <div className="flex gap-2">
          <Button onClick={handleDone} size="sm">Done</Button>
          <Button onClick={handleCancel} size="sm" variant="outline">Cancel</Button>
          <span className="text-sm text-muted-foreground self-center">
            Press keys... ({pressedKeys.length}/3)
          </span>
        </div>
      ) : (
        <div className="flex gap-2">
          <Button onClick={handleStartCapture} size="sm" variant="outline">
            {value.length > 0 ? 'Change Keys' : 'Capture Keys'}
          </Button>
          {value.length > 0 && (
            <Button onClick={() => onChange([])} size="sm" variant="ghost">
              Clear
            </Button>
          )}
        </div>
      )}
    </div>
  )
}
```

### Shortcuts Configuration Component

Create `src/components/preferences/ShortcutsConfig.tsx`:

**Features:**
- Uses TanStack Query for data fetching
- Three `KeyCaptureInput` sections (push-to-record, hands-free-start, hands-free-stop)
- Auto-save on changes (no separate save button needed)
- Reset to defaults button
- Info alert: "Changes take effect immediately - no restart required!"

**Query Hooks:**
```tsx
const { data: config } = useQuery({
  queryKey: ['shortcutsConfig'],
  queryFn: () => commands.loadShortcutsConfig()
})

const saveMutation = useMutation({
  mutationFn: (newConfig: ShortcutsConfig) =>
    commands.saveShortcutsConfig(newConfig),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['shortcutsConfig'] })
  }
})
```

### Update Hotkeys Preferences Page

Modify [Hotkeys.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/preferences/Hotkeys.tsx):

```tsx
import { ShortcutsConfiguration } from '@/components/preferences/ShortcutsConfig'

export function Hotkeys() {
  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold">Keyboard Shortcuts</h2>
        <p className="text-sm text-muted-foreground">
          Configure your recording shortcuts
        </p>
      </div>

      <ShortcutsConfiguration />
    </div>
  )
}
```

Remove the old restart logic - hot-swap makes it unnecessary!

### Update Onboarding

**Option 1: Replace TriggerKeyStep** (Recommended)
- Rename/replace [TriggerKeyStep.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/onboarding/steps/TriggerKeyStep.tsx) with `ShortcutsStep.tsx`
- Use `ShortcutsConfiguration` component (same as preferences)
- Update route in `src/routes/onboarding/trigger-key.tsx` → `shortcuts.tsx`
- Update `OnboardingStep` enum in [config.rs:99](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L99): change `TriggerKey` to `Shortcuts`
- Update step order in onboarding navigation utils

**Option 2: Keep TriggerKeyStep, Add ShortcutsStep Later**
- Keep existing trigger key step for simplicity
- Users can configure full shortcuts in preferences
- Less disruption to onboarding flow

**Recommendation:** Use Option 1 for consistency with the full vision.

---

## Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/shortcuts/events.rs` | KeyCaptureEvent enum for streaming key events to frontend |
| `src-tauri/src/commands/preferences/shortcuts.rs` | Load/save/reset shortcuts + start/stop key capture commands |
| `src/hooks/useKeyCapture.ts` | Hook to listen to KeyCaptureEvent stream from backend |
| `src/components/shortcuts/KeyCaptureInput.tsx` | Interactive key capture component using useKeyCapture hook |
| `src/components/preferences/ShortcutsConfig.tsx` | Shortcuts configuration UI (3 sections) |
| `src/components/onboarding/steps/ShortcutsStep.tsx` | Onboarding step for shortcuts (optional, see above) |
| `src/routes/onboarding/shortcuts.tsx` | Route for shortcuts onboarding step |
| `crates/keyboard/src/key.rs` (add methods) | Add `to_macos_keycode()` and `to_label()` methods to Key enum |

---

## Files to Modify

| File | Changes |
|------|---------|
| [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs) | Add ShortcutKey, Shortcut, ShortcutsConfig structs; add migration function |
| [keyboard_listener.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs) | Add config_tx field; cache shortcuts locally; check config_rx.try_recv(); add update_shortcuts() method; track pressed keys HashSet |
| [setup.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/setup.rs) | Call migration; pass initial config to KeyListener; manage KeyListener in Tauri state; update globe key fix |
| [commands/mod.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/commands/mod.rs) | Add `pub mod shortcuts;` |
| [commands/registry.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/commands/registry.rs) | Register 5 shortcuts commands (load, save, reset, start_capture, stop_capture) |
| [lib.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/lib.rs) | Add `mod shortcuts` and register KeyCaptureEvent in tauri-specta builder |
| [setup.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/setup.rs) | Add `app.manage(KeyCaptureState::new());` |
| [Hotkeys.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/preferences/Hotkeys.tsx) | Replace with ShortcutsConfiguration component; remove restart logic |
| [TriggerKeyStep.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/onboarding/steps/TriggerKeyStep.tsx) | Rename to ShortcutsStep and use ShortcutsConfiguration |
| `crates/keyboard/src/key.rs` | Add `pub fn to_macos_keycode(&self) -> u32` and `pub fn to_label(&self) -> String` methods |
| [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs) OnboardingStep | Change `TriggerKey` to `Shortcuts` (or keep both for migration) |

---

## Implementation Phases

### Phase 1: Backend Foundation (No Breaking Changes)
1. Add `ShortcutKey`, `Shortcut`, `ShortcutsConfig` to `config.rs`
2. Add `ConfigKey::SHORTCUTS` constant
3. Add migration function `migrate_trigger_to_shortcuts()`
4. Add `to_macos_keycode()` to `dictara_keyboard::Key`
5. **Keep old RecordingTrigger** for now (backward compatibility)

**Verification:** Run `npm run verify` - should compile with no errors

### Phase 2: Keyboard Listener Update
1. Update `KeyListener::start()` signature to accept `initial_config: ShortcutsConfig`
2. Create mpsc channel for config updates: `(config_tx, config_rx)`
3. Cache shortcuts locally in spawned thread
4. Implement `config_rx.try_recv()` check on each event
5. Implement pressed keys tracking with `HashSet<u32>`
6. Add shortcut matching logic
7. Add `update_shortcuts()` method
8. Remove hardcoded `LOCK_MODIFIER` constant

**Verification:** Test keyboard events still work with simple shortcuts

### Phase 3: Setup Integration & Migration
1. Update `setup.rs` to call migration on startup
2. Pass initial config to `KeyListener::start()`
3. Manage KeyListener in Tauri state with `app.manage(listener)`
4. Update globe key fix logic to use `KeyListener::uses_fn_key(&config)`
5. Test migration with existing user configs

**Verification:** Existing users' trigger preferences migrate correctly

### Phase 4: Backend Commands
1. Create `commands/preferences/shortcuts.rs`
2. Add `load_shortcuts_config`, `save_shortcuts_config`, `reset_shortcuts_config`
3. Register commands in mod.rs and registry.rs
4. Test hot-swap functionality (save should update without restart)

**Verification:** Commands work via Tauri devtools, hot-swap updates keyboard listener

### Phase 5: Backend - Key Capture Events & Commands
1. Create `src-tauri/src/shortcuts/events.rs` with `KeyCaptureEvent` enum
2. Add `start_key_capture` and `stop_key_capture` commands to shortcuts.rs
3. Add `KeyCaptureState` struct with `Arc<Mutex<bool>>` for controlling capture
4. Register events in lib.rs and manage KeyCaptureState in setup.rs
5. Test event streaming via Tauri devtools

**Verification:** Can start/stop capture, events stream to frontend correctly

### Phase 6: Frontend - Key Capture Hook & Component
1. Create `useKeyCapture.ts` hook to listen to KeyCaptureEvent stream
2. Create `KeyCaptureInput.tsx` using the hook
3. Test capturing various key combinations including Fn key
4. Verify real-time visual feedback

**Verification:** Can capture Shift+R, Cmd+Space, Fn, Fn+Space, etc. via backend streaming

### Phase 7: Frontend - Shortcuts Configuration UI
1. Create `ShortcutsConfig.tsx` with TanStack Query hooks
2. Add three KeyCaptureInput sections
3. Implement auto-save on change
4. Add reset to defaults button

**Verification:** Can configure all 3 shortcuts, changes save and take effect immediately

### Phase 8: Preferences Integration
1. Update `Hotkeys.tsx` to use new component
2. Remove old restart logic and TriggerKeySelector
3. Test in preferences page

**Verification:** Preferences page shows new UI, shortcuts work, no restart needed

### Phase 9: Onboarding Update (Optional)
1. Create `ShortcutsStep.tsx` or update `TriggerKeyStep.tsx`
2. Update routes and navigation
3. Update `OnboardingStep` enum in config.rs
4. Test onboarding flow

**Verification:** Onboarding works with new shortcuts step

### Phase 10: Testing & Edge Cases
1. Test overlapping shortcuts (e.g., Fn vs Fn+Space)
2. Test rapid key presses and releases
3. Test hot-swap while recording
4. Test migration from all old trigger types
5. Test Fn key special handling
6. Run `npm run verify`

**Verification:** All edge cases handled correctly, no crashes

### Phase 11: Cleanup (Optional)
1. Consider removing old `RecordingTrigger` (breaking change)
2. Remove old `TriggerKeySelector` component if unused
3. Update documentation

---

## Key Technical Considerations

### 1. Key Combination Matching Algorithm

**Problem:** How to match exact key combinations without false positives?

**Solution:** HashSet-based exact matching
- Track all pressed keys: `pressed_keys: HashSet<u32>`
- On key press: add to set, check matches
- On key release: check matches FIRST, then remove from set
- Match condition: `shortcut.keys.len() == pressed_keys.len() && all_present`

**Why this works:**
- Prevents subset matches (Fn won't match Fn+Space)
- Order-independent (Shift+R == R+Shift)
- O(1) lookup with HashSet
- Race condition safe (check before removing)

### 2. Hot-Swap Mechanism

**Problem:** Keyboard listener runs in separate thread, how to update without restart?

**Solution:** mpsc channel-based config updates
- KeyListener creates channel: `(config_tx, config_rx)`
- **Local cache**: shortcuts stored as `mut shortcuts` variable in thread
- Event loop checks `config_rx.try_recv()` on each event (~5ns overhead)
- `update_shortcuts()` method sends new config via `config_tx.blocking_send()`
- No locks needed - just a simple variable update!

**Performance:** ~5ns overhead per event (faster than RwLock!)

### 3. Keycode Mismatch Prevention

**Problem:** JavaScript keycodes ≠ macOS keycodes

**Solution:** Backend-streamed key capture
- Backend uses same `dictara_keyboard` crate as runtime listener
- Events contain correct macOS keycodes (e.g., Space = 49, not 32)
- Frontend just displays and saves what backend sends
- **Bonus:** Can capture Fn key that JavaScript cannot detect!

### 4. Space Key Swallowing

**Current behavior:** Space is swallowed when it locks recording

**New behavior:** Only swallow Space if it's part of `hands_free_start` shortcut
- Check: `hands_free_start.keys.iter().any(|k| k.keycode == 49)`
- Allows Space in other contexts (e.g., Ctrl+Space for trigger)

### 5. Migration Strategy

**Non-destructive migration:**
1. Check if `shortcutsConfig` exists → skip migration
2. Load old `recordingTrigger` from `appConfig`
3. Convert to new 3-shortcut format (preserving user's trigger choice)
4. Save to `shortcutsConfig`
5. **Keep old config** for rollback safety

**Backward compatibility:** Default shortcuts match old behavior exactly

### 6. Globe Key Fix Dynamic Update

**Current:** Globe key fix applied once at startup if using Fn

**New:** Update dynamically when shortcuts change
- Track old vs new Fn usage in `save_shortcuts_config`
- Only call `fix_globe_key_if_needed()` when transitioning from no-Fn to Fn
- Leave setting unchanged when removing Fn (safer UX)

### 7. Validation

**Backend validation (critical):**
- 1-3 keys per shortcut
- No duplicate keys within a shortcut
- Valid keycodes (0-127 range)
- Return clear error messages

**Frontend validation (UX):**
- Disable capture button after 3 keys
- Show error toast on save failure
- Prevent clearing required shortcuts

### 8. Thread Safety

**Why mpsc channel over Arc<RwLock>:**
- **Simpler**: No lock management needed
- **Faster**: Local variable read vs RwLock acquisition
- **More idiomatic**: Channels are the Rust way for thread communication
- **No contention**: try_recv() is non-blocking and lock-free

**Why this works:**
- Keyboard events: ~50-100/sec → fast local reads
- Config updates: ~1/hour → rare channel sends
- `ShortcutsConfig` is small (~100 bytes) → cheap to clone through channel
- No shared mutable state → no synchronization bugs possible

---

## Edge Cases & Mitigations

| Edge Case | Mitigation |
|-----------|------------|
| Overlapping shortcuts (Fn vs Fn+Space) | Exact length matching prevents subset matches |
| Same shortcut for multiple actions | Backend validation warns about conflicts |
| Fn key can't be captured by JS | Special "Use Fn" button in UI |
| User holds 4+ keys | Frontend limits to 3 keys max |
| Rapid key presses | HashSet ensures correct state tracking |
| Hot-swap during recording | State manager handles gracefully, recording continues |
| Migration fails | Config falls back to defaults (Fn trigger) |
| Invalid keycodes from frontend | Backend validation rejects and returns error |
| Thread deadlock | No locks used - channel communication is deadlock-free |
| Globe key fix race condition | Only update on Fn usage change, not every save |

---

## Verification & Testing

### Manual Testing Checklist

**Backend:**
- [ ] Migration from all 4 old trigger types (Fn, Control, Option, Command)
- [ ] Hot-swap: change shortcuts without restart, verify they work immediately
- [ ] Validation: try saving invalid shortcuts (0 keys, 4 keys, duplicates)
- [ ] Globe key fix: verify updates when Fn usage changes

**Frontend:**
- [ ] Key capture: capture Shift+R, Cmd+Space, Control+Option+F
- [ ] Fn button: add Fn key to shortcuts
- [ ] Max keys: verify cannot add 4th key
- [ ] Remove keys: click X button to remove individual keys
- [ ] Clear: clear all keys and re-capture
- [ ] Reset: reset to defaults button works

**Integration:**
- [ ] Push-to-record: hold keys, release stops recording
- [ ] Hands-free start: press combo, recording locks
- [ ] Hands-free stop: press combo, recording stops
- [ ] Overlapping shortcuts: Fn and Fn+Space work correctly (no false triggers)
- [ ] Space swallowing: Space only swallowed in hands-free-start combo
- [ ] Onboarding: new shortcuts step works, saves config

**Edge Cases:**
- [ ] Press keys in different orders (Shift+R vs R+Shift)
- [ ] Rapid press/release cycles
- [ ] Change shortcuts while recording (should handle gracefully)
- [ ] Migrate from old config, verify preserved trigger choice
- [ ] Multiple users with different configs (no cross-contamination)

### Automated Tests

**Backend (Rust):**
```rust
#[test]
fn test_shortcut_matching() {
    let shortcut = Shortcut {
        keys: vec![
            ShortcutKey { keycode: 56, label: "Shift" },
            ShortcutKey { keycode: 17, label: "R" },
        ],
    };

    let mut pressed = HashSet::new();
    pressed.insert(56);
    pressed.insert(17);
    assert!(shortcut.matches(&pressed));

    // Subset should not match
    pressed.remove(&17);
    assert!(!shortcut.matches(&pressed));
}

#[test]
fn test_migration() {
    let store = MockConfigStore::new();

    // Set old trigger
    store.set(&ConfigKey::APP, AppConfig {
        recording_trigger: RecordingTrigger::Control,
        ..Default::default()
    });

    // Run migration
    migrate_trigger_to_shortcuts(&store).unwrap();

    // Verify new config
    let shortcuts = store.get(&ConfigKey::SHORTCUTS).unwrap();
    assert_eq!(shortcuts.push_to_record.keys[0].keycode, 59); // Control
}
```

**Frontend (TypeScript/Vitest):**
```tsx
test('KeyCaptureInput captures keys', async () => {
  const onChange = vi.fn()
  render(<KeyCaptureInput value={[]} onChange={onChange} />)

  fireEvent.click(screen.getByText('Capture Keys'))
  fireEvent.keyDown(window, { key: 'Shift', keyCode: 56 })

  expect(onChange).toHaveBeenCalledWith([
    { keycode: 56, label: 'Shift' }
  ])
})
```

### Build Verification

Run after each phase:
```bash
npm run verify
```

This runs:
- Rust compilation and tests
- TypeScript type checking
- Tauri-specta binding generation
- Frontend build

All must pass before proceeding to next phase.

---

## Success Criteria

- [ ] Existing users seamlessly migrated from simple trigger to 3-shortcut system
- [ ] All three shortcuts configurable with 1-3 key combinations
- [ ] Changes take effect immediately without app restart
- [ ] Key capture UI works intuitively (including Fn button)
- [ ] No regressions in recording functionality
- [ ] `npm run verify` passes completely
- [ ] Manual testing checklist 100% complete
- [ ] Code follows existing patterns (ConfigStore, tauri-specta, TanStack Query)

---

## Rollback Plan

If critical issues are discovered:

1. **Phase 1-3 (Backend only):** Revert commits, migration is backward compatible
2. **Phase 4-7 (Commands + Frontend):** Keep backend, disable frontend UI, fall back to simple trigger selector
3. **Phase 8+ (Onboarding):** Keep old TriggerKeyStep, don't activate ShortcutsStep

Migration is non-destructive - old `appConfig.recordingTrigger` is preserved.

---

## Future Enhancements (Out of Scope)

- Dynamic shortcut labels in onboarding tutorials (FnHoldStep, FnSpaceStep)
- Conflict detection warning when shortcuts overlap
- Export/import shortcuts config
- Shortcut presets (Vim mode, Emacs mode, etc.)
- Global keyboard listener (work in any app, not just when focused)
- Visual keyboard layout for configuration
- Accessibility: screen reader support for key capture

These can be added incrementally after the core feature is stable.
