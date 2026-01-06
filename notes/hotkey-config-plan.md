# Plan: Make Recording Trigger Configurable

This document outlines how to make the recording trigger user-configurable and expose it in Preferences with a “Hotkeys” tab.

## Goal
- Let users pick the key used to start/stop recording (default: Fn).
- Apply changes without requiring app restart.
- Allow simple in-app testing/confirmation.

## UX Additions (Frontend)
- Add a new Preferences sidebar entry: “Hotkeys” (or “Shortcuts”).
- Route: `/preferences/hotkeys` with a `Hotkeys` component.
- Form fields:
  - Dropdown of allowed triggers (e.g., Fn, Control, Option, Command, Caps Lock).
  - Optional “Press a key to set” capture button to listen for the next key press (via a short-lived Tauri command).
  - Status text showing the current trigger and whether it is active (e.g., “Listening for Control press/release”).
- Actions:
  - “Save” (persists via `saveAppConfig`/new command).
  - “Test” (optional): start a temporary listener and show live feedback when the trigger is pressed/released.
- Update `PreferencesLayout` menu to include the new tab.
- Regenerate route tree if needed (depending on router setup).

## Data Model & Commands (Backend)
- In `src-tauri/src/config.rs`:
  - Add a `RecordingTrigger` enum (`Fn`, `Control`, `Option`, `Command`, `CapsLock`, etc.) with serde + specta derives and camelCase serialization.
  - Add a `recording_trigger: RecordingTrigger` field to `AppConfig` with a default of `Fn`.
- In `src-tauri/src/tauri_commands.rs`:
  - Extend `load_app_config`/`save_app_config` to include `recording_trigger`.
  - Optionally add a dedicated `set_recording_trigger` command that also updates the live listener (see below).
- Regenerate `src/bindings.ts` (tauri-specta) in dev so the frontend picks up the new types.

## Keyboard Listener Changes
- `src-tauri/src/keyboard_listener.rs`:
  - Replace the `RECORDING_TRIGGER` const with a value read from config.
  - Keep the trigger in a shared `Arc<RwLock<Key>>` so it can be updated at runtime.
  - Update the grab closure to read the current trigger on each event (or cache locally but refresh on change).
  - Keep `LOCK_MODIFIER` (Space) behavior unchanged.
- Live updates:
  - When `save_app_config` (or `set_recording_trigger`) runs, also update the `RwLock` so the running listener uses the new key without restart.
  - If runtime swap is complex, fallback: store the trigger and restart the listener thread when it changes.

## macOS Keyboard Event Support
- In `crates/keyboard/src/macos.rs`:
  - Currently only Fn gets press/release via FlagsChanged. Extend state tracking for other modifiers you plan to allow (Control, Option, Command, CapsLock) so both press and release are emitted; otherwise Control will start but never stop recording.
- In `crates/keyboard/src/key.rs`:
  - Ensure all chosen triggers map correctly from macOS keycodes (ControlLeft/Right, Option, Command, CapsLock, Fn already present).

## App Setup Wiring
- In `src-tauri/src/setup.rs`:
  - Load `recording_trigger` from config and pass it into `KeyListener::start`.
  - Only run `globe_key::fix_globe_key_if_needed()` when trigger is `Fn`; skip for others.
  - Store the trigger handle (RwLock) in state if commands need to update it at runtime.

## Frontend Data Flow
- Update `useAppConfig`/`useSaveAppConfig` types once bindings regenerate.
- Build `Hotkeys` component:
  - Load current `recording_trigger` via `useAppConfig`.
  - Let users pick from the dropdown or capture the next key (invoke new Tauri command to read one key press and return the `RecordingTrigger` variant).
  - Persist via `saveAppConfig` (or `set_recording_trigger` if added) and show success/error states.
  - Optional “Test”: call a lightweight command to start a short-lived listener and stream back events, or simply display the current trigger name and instruct the user to press it (listening to a quick command that returns true/false).

## Onboarding Copy
- Audit Fn-specific onboarding steps (`FnHoldStep`, `FnSpaceStep`) and update text/visuals to reflect either:
  - The current trigger if set, or
  - A note that Fn is the default and can be changed later in Preferences.

## Validation
- Manual checks:
  - Default Fn still works and emoji picker stays suppressed (via globe key fix).
  - Switch to Control/Command/etc., press/release starts and stops recording; no stuck state.
  - Preferences tab saves and reloads the chosen trigger.
  - Switching triggers does not require app restart.
- Automated ideas:
  - Add unit tests in `crates/keyboard` for modifier press/release state tracking.
  - Round-trip test for `RecordingTrigger` serde default in config.

## Notes & Edge Cases
- Ensure only one modifier is allowed at a time (keep UX simple).
- Consider disallowing keys that are likely to conflict with system shortcuts.
- If capture mode is added, swallow the capture key during selection to avoid side effects.
