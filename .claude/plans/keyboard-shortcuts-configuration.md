# Keyboard Shortcuts Configuration Feature - Implementation Plan

## Summary
Add a configurable keyboard shortcuts system to Dictara, replacing hardcoded Fn and Space key constants with user-configurable shortcuts. Includes a new Preferences page, a new onboarding step, and dynamic UI text that reflects user's chosen shortcuts.

## User Requirements
1. **Three configurable shortcuts:**
   - **Push to Record** (default: Fn) - hold to record, release to transcribe
   - **Hands-free Record** (default: Fn + Space) - toggle recording on
   - **Stop Hands-free Record** (default: Fn) - stop hands-free recording

2. **Key combinations supported** - up to 3 keys per shortcut (e.g., Shift+Cmd+R)
3. **Preferences page** - "Shortcuts" tab with UI showing key badges (like screenshot)
4. **Onboarding step** - after API Keys, before tutorials
5. **Dynamic UI text** - all "Fn" references should show the configured shortcut
6. **Always configure Globe key** - regardless of chosen keys

---

## Files to Create

### Backend (Rust)
| File | Purpose |
|------|---------|
| `src-tauri/src/shortcuts_state.rs` | Thread-safe runtime state for shortcuts config |

### Frontend (TypeScript/React)
| File | Purpose |
|------|---------|
| `src/routes/preferences/shortcuts.tsx` | Route for shortcuts preference page |
| `src/routes/onboarding/shortcuts.tsx` | Route for shortcuts onboarding step |
| `src/components/preferences/Shortcuts.tsx` | Shortcuts preference page component |
| `src/components/preferences/shortcuts/ShortcutInput.tsx` | Key combination capture component |
| `src/components/preferences/shortcuts/KeyBadge.tsx` | Individual key badge component (like "⇧ Shift") |
| `src/components/onboarding/steps/ShortcutsStep.tsx` | Shortcuts onboarding step |
| `src/hooks/useShortcutsConfig.ts` | React Query hooks for shortcuts config |
| `src/contexts/ShortcutLabelsContext.tsx` | Context for dynamic shortcut labels |

---

## Files to Modify

### Backend (Rust)
| File | Changes |
|------|---------|
| [config.rs](src-tauri/src/config.rs) | Add `ShortcutsConfig`, `Shortcut` structs, load/save functions |
| [keyboard_listener.rs](src-tauri/src/keyboard_listener.rs) | Replace constants with runtime config, support key combinations |
| [setup.rs](src-tauri/src/setup.rs) | Initialize `ShortcutsConfigState`, pass to `KeyListener::start()` |
| [tauri_commands.rs](src-tauri/src/tauri_commands.rs) | Add `load_shortcuts_config`, `save_shortcuts_config`, `get_key_label` commands |
| [lib.rs](src-tauri/src/lib.rs) | Register new commands and `ShortcutsConfigState` |
| [crates/keyboard/src/key.rs:159](crates/keyboard/src/key.rs#L159) | Change `pub(crate)` to `pub` for `from_macos_keycode` |

### Frontend (TypeScript/React)
| File | Changes |
|------|---------|
| [PreferencesLayout.tsx](src/components/preferences/PreferencesLayout.tsx) | Add "Shortcuts" menu item |
| [useOnboardingNavigation.ts](src/hooks/useOnboardingNavigation.ts) | Add `shortcuts` step to `STEP_ORDER` and `STEP_ROUTES` |
| [utils.ts](src/components/onboarding/utils.ts) | Add `shortcuts` step definition, update step labels |
| [FnHoldStep.tsx](src/components/onboarding/steps/FnHoldStep.tsx) | Use `useShortcutLabels()` for dynamic shortcut display |
| [FnSpaceStep.tsx](src/components/onboarding/steps/FnSpaceStep.tsx) | Use `useShortcutLabels()` for dynamic shortcut display |
| [AccessibilityStep.tsx](src/components/onboarding/steps/AccessibilityStep.tsx) | Use `useShortcutLabels()` for dynamic shortcut display |
| [CompleteStep.tsx](src/components/onboarding/steps/CompleteStep.tsx) | Use `useShortcutLabels()` for dynamic shortcut display |
| [KeyboardVisual.tsx](src/components/onboarding/KeyboardVisual.tsx) | Support dynamic shortcut display with multiple keys |

---

## Implementation Phases

### Phase 1: Backend Foundation
1. Add `Shortcut` and `ShortcutsConfig` structs to `config.rs`:
   ```rust
   /// A keyboard shortcut (1-3 keys)
   #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
   #[serde(rename_all = "camelCase")]
   pub struct Shortcut {
       /// Array of macOS keycodes (1-3 keys)
       pub keys: Vec<u32>,
   }

   /// Shortcuts configuration for recording actions
   #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
   #[serde(rename_all = "camelCase")]
   pub struct ShortcutsConfig {
       /// Push to record - hold to record, release to transcribe (default: Fn)
       pub push_to_record: Shortcut,
       /// Hands-free record - toggle recording on (default: Fn + Space)
       pub hands_free_record: Shortcut,
       /// Stop hands-free - stop hands-free recording (default: Fn)
       pub stop_hands_free: Shortcut,
   }

   impl Default for ShortcutsConfig {
       fn default() -> Self {
           Self {
               push_to_record: Shortcut { keys: vec![63] },        // Fn
               hands_free_record: Shortcut { keys: vec![63, 49] }, // Fn + Space
               stop_hands_free: Shortcut { keys: vec![63] },       // Fn
           }
       }
   }
   ```
2. Add `keycode_to_label(keycode: u32) -> String` utility
3. Add `shortcut_to_display(shortcut: &Shortcut) -> String` utility (returns "Fn + Space")
4. Add `load_shortcuts_config` and `save_shortcuts_config` functions
5. Create `shortcuts_state.rs` with `ShortcutsConfigState` (uses `RwLock` for thread-safe access)
6. Add Tauri commands: `load_shortcuts_config`, `save_shortcuts_config`, `get_key_label`
7. Register commands in `lib.rs`
8. Make `Key::from_macos_keycode` public in `crates/keyboard/src/key.rs`

### Phase 2: Keyboard Listener Updates
1. Update `KeyListener::start()` to accept `Arc<ShortcutsConfigState>` parameter
2. Track currently held keys (HashSet of keycodes)
3. Match shortcuts by checking if all keys in shortcut are currently held:
   ```rust
   fn matches_shortcut(held_keys: &HashSet<u32>, shortcut: &Shortcut) -> bool {
       shortcut.keys.iter().all(|k| held_keys.contains(k))
   }
   ```
4. Update `setup.rs` to:
   - Load shortcuts config from store
   - Create `ShortcutsConfigState`
   - Manage state with `app.manage()`
   - Pass state to `KeyListener::start()`

### Phase 3: Preferences UI
1. Create `useShortcutsConfig.ts` hooks:
   - `useShortcutsConfig()` - load config
   - `useSaveShortcutsConfig()` - save config
2. Create `KeyBadge.tsx` component:
   - Displays a single key as a badge (like "⇧ Shift" in the screenshot)
   - Props: `keycode`, `size`
3. Create `ShortcutInput.tsx` component:
   - Shows current shortcut as key badges
   - Click edit button to enter "listening" mode
   - Capture all held keys when user presses (up to 3)
   - Special "Use Fn" button (Fn can't be captured by JS)
   - Display: `[⇧ Shift] [r] [✎]` (edit button)
4. Create `Shortcuts.tsx` preference page with 3 inputs
5. Create `shortcuts.tsx` route
6. Add menu item to `PreferencesLayout.tsx`

### Phase 4: Onboarding Integration
1. Add `Shortcuts` variant to `OnboardingStep` enum in `config.rs`
2. Create `ShortcutsStep.tsx` component (simplified version of preferences)
3. Create `onboarding/shortcuts.tsx` route
4. Update `useOnboardingNavigation.ts`:
   - Add `'shortcuts'` to `STEP_ORDER` after `'api_keys'`
   - Add route mapping
5. Update step labels in `utils.ts`

### Phase 5: Dynamic UI Text
1. Create `ShortcutLabelsContext.tsx`:
   - Provides formatted shortcut strings for each action
   - `pushToRecord`: "Fn" or "Shift + R"
   - `handsFreeRecord`: "Fn + Space" or "Cmd + Space"
   - `stopHandsFree`: "Fn"
2. Update onboarding steps to use `useShortcutLabels()`:
   - `FnHoldStep.tsx` - display `pushToRecord` shortcut
   - `FnSpaceStep.tsx` - display `handsFreeRecord` shortcut
   - `AccessibilityStep.tsx` - reference configured shortcut
   - `CompleteStep.tsx` - show summary with actual shortcuts
3. Update `KeyboardVisual.tsx` to render multiple key badges

### Phase 6: Testing & Verification
1. Run `npm run verify`
2. Test shortcut changes in preferences
3. Test key combinations (2 and 3 keys)
4. Test new onboarding step flow
5. Verify keyboard listener matches combinations correctly
6. Check all UI text updates correctly

---

## Key Technical Details

### Key Codes (macOS)
| Key | Code | Display |
|-----|------|---------|
| Fn | 63 | "Fn" |
| Space | 49 | "Space" |
| Shift (Left) | 56 | "⇧ Shift" |
| Shift (Right) | 60 | "⇧ Shift" |
| Cmd (Left) | 55 | "⌘ Cmd" |
| Cmd (Right) | 54 | "⌘ Cmd" |
| Option (Left) | 58 | "⌥ Option" |
| Option (Right) | 61 | "⌥ Option" |
| Control (Left) | 59 | "⌃ Control" |
| Control (Right) | 62 | "⌃ Control" |
| F1-F12 | 122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111 | "F1"-"F12" |
| Return | 36 | "Return" |
| Tab | 48 | "Tab" |
| Escape | 53 | "Esc" |

### Thread Safety
The keyboard listener runs in a separate thread. Use `RwLock<ShortcutsConfig>` in `ShortcutsConfigState` for safe concurrent access:
- Keyboard listener reads config frequently
- Preferences page writes config occasionally
- No restart required when config changes

### Key Combination Matching
For shortcuts with multiple keys:
1. Track all currently held keys in a `HashSet<u32>`
2. On key press, add to set; on release, remove from set
3. Check if shortcut matches: `shortcut.keys.iter().all(|k| held_keys.contains(k))`
4. Order of keys in shortcut doesn't matter (Shift+R == R+Shift)

### Fn Key Special Handling
JavaScript cannot capture the Fn key in a webview. The `ShortcutInput` component needs:
- A dedicated "Use Fn" button that adds keycode 63 to the shortcut
- Clear indication when Fn is part of the current shortcut

### UI Design (like screenshot)
```
Push to Record                    [⇧ Shift] [r]  [✎]
Hold to say something short
```
- Key badges in a row, styled like the screenshot
- Edit button (pencil icon) to start capture mode
- When capturing: "Press keys..." with cancel option

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Fn key can't be captured by JS | Add explicit "Use Fn" button in UI |
| Same shortcut for multiple actions | Add validation to warn about conflicts |
| Overlapping shortcuts (Fn vs Fn+Space) | Check for most specific match first (longer shortcuts) |
| Config changes while recording | Only apply config on next keypress (current design handles this) |
| Backward compatibility | Default config matches current hardcoded behavior |
| Too many keys pressed (>3) | Limit to first 3 keys captured |
