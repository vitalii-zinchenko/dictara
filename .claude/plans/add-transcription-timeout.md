# Plan: Add Transcription Timeout for All Providers

## Summary
Add a 20-second timeout for transcription across all providers (OpenAI, Azure OpenAI, Local Model) to prevent indefinite hangs when transcription takes too long.

## Configuration
- **Timeout value**: 20 seconds (hardcoded constant)
- **Scope**: Same for all providers

---

## Implementation Steps

### Step 1: Add Timeout Constant
**File**: [src-tauri/src/clients/transcriber.rs](src-tauri/src/clients/transcriber.rs)

Add a new constant alongside existing ones:
```rust
/// Timeout for transcription requests (applies to all providers)
const TRANSCRIPTION_TIMEOUT_SECS: u64 = 20;
```

### Step 2: Add Timeout Error Variant
**File**: [src-tauri/src/clients/error.rs](src-tauri/src/clients/error.rs)

Add new error variant:
```rust
#[error("Transcription timed out after {0} seconds")]
TranscriptionTimeout(u64),
```

Add user-friendly message in `user_message()`:
```rust
TranscriptionError::TranscriptionTimeout(_) => {
    "Transcription took too long. Try again.".to_string()
}
```

### Step 3: Update API Transcriber (OpenAI & Azure)
**File**: [src-tauri/src/clients/api_transcriber.rs](src-tauri/src/clients/api_transcriber.rs)

Change line 33 from:
```rust
let http_client = reqwest::blocking::Client::new();
```

To:
```rust
use std::time::Duration;
use super::transcriber::TRANSCRIPTION_TIMEOUT_SECS;

let http_client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS))
    .build()
    .map_err(|e| TranscriptionError::ApiError(e.to_string()))?;
```

Update error handling to detect timeout:
```rust
Err(e) => {
    if e.is_timeout() {
        Err(TranscriptionError::TranscriptionTimeout(TRANSCRIPTION_TIMEOUT_SECS))
    } else {
        Err(TranscriptionError::ApiError(e.to_string()))
    }
}
```

### Step 4: Update Local Model Transcription
**File**: [src-tauri/src/models/local_client.rs](src-tauri/src/models/local_client.rs)

Modify `transcribe_file` to accept timeout and use abort callback:

```rust
use std::time::{Duration, Instant};

pub fn transcribe_file(
    &self,
    audio_path: &Path,
    timeout: Duration,
) -> Result<String, TranscriptionError> {
    // ... existing audio loading code ...

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Set abort callback to enforce timeout
    let start_time = Instant::now();
    params.set_abort_callback_safe(move || {
        start_time.elapsed() > timeout
    });

    // ... rest of transcription ...

    // After state.full() call, check if we aborted due to timeout
    // whisper-rs returns an error when aborted, map it to our timeout error
}
```

### Step 5: Update ModelLoader to Pass Timeout
**File**: [src-tauri/src/models/loader.rs](src-tauri/src/models/loader.rs)

Update `transcribe_with_model` (line 269-272):
```rust
model.client
    .transcribe_file(audio_path, Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS))
    .map_err(|e| e.to_string())
```

Also update the `transcribe` method (line 216-219) similarly.

### Step 6: Update LocalTranscriber
**File**: [src-tauri/src/clients/local_transcriber.rs](src-tauri/src/clients/local_transcriber.rs)

The `LocalTranscriber` calls `self.loader.transcribe_with_model()` which will now handle timeout internally. No changes needed here unless we want to pass timeout as a parameter.

---

## Files to Modify

| File | Change |
|------|--------|
| [src-tauri/src/clients/transcriber.rs](src-tauri/src/clients/transcriber.rs) | Add `TRANSCRIPTION_TIMEOUT_SECS` constant |
| [src-tauri/src/clients/error.rs](src-tauri/src/clients/error.rs) | Add `TranscriptionTimeout` error variant |
| [src-tauri/src/clients/api_transcriber.rs](src-tauri/src/clients/api_transcriber.rs) | Add HTTP client timeout, detect timeout errors |
| [src-tauri/src/models/local_client.rs](src-tauri/src/models/local_client.rs) | Add timeout param, use `set_abort_callback_safe` |
| [src-tauri/src/models/loader.rs](src-tauri/src/models/loader.rs) | Pass timeout to `transcribe_file` calls |

---

## Error Flow

```
Timeout occurs
    |
    v
TranscriptionError::TranscriptionTimeout(20)
    |
    v
user_message() -> "Transcription took too long. Try again."
    |
    v
RecordingStateChanged::Error event emitted to frontend
    |
    v
User sees friendly error message with option to retry
```

---

## Verification

1. **Build**: Run `npm run verify` to ensure no compilation errors
2. **Test API timeout**:
   - Temporarily set timeout to 1 second
   - Record audio and trigger transcription
   - Verify timeout error is shown with user-friendly message
3. **Test Local model timeout**:
   - Same as above but with Local provider selected
   - Verify abort callback properly stops transcription
4. **Test normal operation**:
   - Reset timeout to 20 seconds
   - Verify short recordings transcribe successfully
   - Verify longer recordings (approaching 20s processing time) still work

---

## Notes

- The `set_abort_callback_safe` in whisper-rs is called periodically during processing, so timeout granularity depends on how often whisper calls it
- For API providers, reqwest timeout covers both connection and total request time
- If abort callback returns `true`, whisper-rs will stop processing and return an error

---

## Implementation Summary

### Remote Providers (OpenAI & Azure) ✅

**Status**: Fully implemented and working

- Added HTTP client timeout using `reqwest::blocking::Client::builder().timeout()`
- Timeout detection via `e.is_timeout()` check
- Returns `TranscriptionError::TranscriptionTimeout(20)` on timeout
- User sees: "Transcription took too long. Try again."

### Local Model Provider ✅

**Status**: Fully implemented and working (with patched whisper-rs)

**Implementation**: Uses `set_abort_callback_safe` with `Arc<AtomicBool>` pattern in [src-tauri/src/models/local_client.rs](src-tauri/src/models/local_client.rs):

```rust
let start_time = Instant::now();
let timed_out = Arc::new(AtomicBool::new(false));
let timed_out_clone = timed_out.clone();

params.set_abort_callback_safe(move || {
    let elapsed = start_time.elapsed();
    if elapsed > timeout {
        timed_out_clone.store(true, Ordering::SeqCst);
        true // Abort transcription
    } else {
        false // Continue
    }
});

// Run transcription
let result = state.full(params, &samples);

// Check if we timed out
if timed_out.load(Ordering::SeqCst) {
    return Err(TranscriptionError::TranscriptionTimeout(timeout.as_secs()));
}
```

---

## whisper-rs Bug Fix

### Bug Discovery

The original `set_abort_callback_safe` implementation in whisper-rs had a **type mismatch bug** that caused captured closure values to appear as zeros.

**Root cause** in [../../whisper-rs (at /Users/vitaliizinchenko/Projects/whisper-rs)/src/whisper_params.rs:631](../../whisper-rs (at /Users/vitaliizinchenko/Projects/whisper-rs)/src/whisper_params.rs#L631):

```rust
// BUG (before):
self.fp.abort_callback = Some(trampoline::<F>);
// Stores Box<Box<dyn FnMut() -> bool>> but trampoline expects F

// FIXED (after):
self.fp.abort_callback = Some(trampoline::<Box<dyn FnMut() -> bool>>);
// Now trampoline type matches what's actually stored
```

### What is a "trampoline"?

A **trampoline function** is a naming convention for FFI (Foreign Function Interface) callback bridging. When Rust needs to pass a closure to C code (like whisper.cpp), the trampoline:

1. Receives the raw C callback with a `void*` user_data pointer
2. Casts user_data back to the correct Rust type
3. Invokes the actual Rust closure

### Why the bug caused zeros

The trampoline function interprets the `user_data` pointer based on its generic type parameter:
- Code stored: `Box<Box<dyn FnMut() -> bool>>`
- Trampoline expected: `F` (the original closure type, different memory layout)

When the types don't match, the trampoline interprets wrong memory locations, causing our `Instant` and `Duration` to appear as zeros.

### Fix Applied

We forked whisper-rs to [../../whisper-rs (at /Users/vitaliizinchenko/Projects/whisper-rs)](../../whisper-rs (at /Users/vitaliizinchenko/Projects/whisper-rs)) and applied a **one-line fix** at line 631 of `src/whisper_params.rs`.

**Cargo.toml** now uses the local patched fork:
```toml
whisper-rs = { path = "../../../whisper-rs (at /Users/vitaliizinchenko/Projects/whisper-rs)", features = ["metal"] }
```

**Workspace** excludes the fork to avoid conflicts:
```toml
exclude = ["ai-context"]
```

### Upstream Status

- Bug exists in latest whisper-rs v0.15.1 (as of January 2026)
- Similar bug in `set_progress_callback_safe` was fixed in PR #220, but abort callback was missed
- No existing bug report found for this specific issue
