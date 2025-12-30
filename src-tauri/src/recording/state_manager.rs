//! Recording State Machine - Single source of truth for valid state transitions
//!
//! State diagram:
//! ```text
//! Ready ──FnDown──> Recording ──FnUp──> Transcribing ──Complete──> Ready
//!   │                   │                    │
//! [Retry]            [Lock]               [Failed]
//!   │                   ↓                    ↓
//!   │          RecordingLocked             Ready (keeps audio for retry)
//!   │                   │
//!   │               [FnDown]──> Transcribing
//!   │               [Cancel]──> Ready
//!   └──────────────────────────> Transcribing
//! ```

use std::sync::Mutex;

/// Events that can trigger state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum RecordingEvent {
    /// Fn key pressed
    FnDown,
    /// Fn key released
    FnUp,
    /// Space key pressed (lock recording)
    Lock,
    /// Escape or cancel action
    Cancel,
    /// Retry transcription with existing audio file
    Retry,
    /// Transcription completed successfully
    TranscriptionComplete,
    /// Transcription failed
    TranscriptionFailed,
}

/// Actions the Controller should perform after a state transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingAction {
    /// Start a new recording session
    StartRecording,
    /// Stop recording and begin transcription
    StopAndTranscribe,
    /// Cancel recording without transcription
    CancelRecording,
    /// Retry transcription with existing audio file
    RetryTranscription,
}

/// Recording states
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum RecordingState {
    /// Controller is ready to start recording
    Ready,
    /// Controller is currently recording
    Recording,
    /// Recording is locked - Fn release will be ignored
    RecordingLocked,
    /// Audio is being transcribed
    Transcribing,
}

impl RecordingState {
    /// Check if the app is busy (recording, locked, or transcribing)
    pub fn is_busy(self) -> bool {
        self != Self::Ready
    }
}

/// Result of a successful state transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionResult {
    /// State changed
    Changed {
        from: RecordingState,
        to: RecordingState,
        action: Option<RecordingAction>,
    },
    /// Event was valid but state didn't change
    Unchanged,
}

/// Reason a transition was rejected
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{attempted_event} event rejected in {current_state} state")]
pub struct TransitionRejection {
    pub current_state: RecordingState,
    pub attempted_event: RecordingEvent,
}

/// Thread-safe recording state manager
#[derive(Debug)]
pub struct RecordingStateManager {
    state: Mutex<RecordingState>,
}

impl RecordingStateManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RecordingState::Ready),
        }
    }

    /// Get the current state (read-only, thread-safe)
    pub fn current(&self) -> RecordingState {
        *self.state.lock().unwrap()
    }

    /// Check if the app is busy (recording or locked)
    pub fn is_busy(&self) -> bool {
        self.current().is_busy()
    }

    /// Attempt a state transition based on an event
    ///
    /// This is the ONLY way to change state - ensures all transitions are valid.
    pub fn transition(
        &self,
        event: RecordingEvent,
    ) -> Result<TransitionResult, TransitionRejection> {
        let mut state = self.state.lock().unwrap();
        let current = *state;

        match self.compute_transition(current, event) {
            Some((new_state, action)) => {
                if new_state == current {
                    return Ok(TransitionResult::Unchanged);
                }

                *state = new_state;
                Ok(TransitionResult::Changed {
                    from: current,
                    to: new_state,
                    action,
                })
            }
            None => Err(TransitionRejection {
                current_state: current,
                attempted_event: event,
            }),
        }
    }

    /// Pure function: compute what transition should happen (if any)
    /// Returns None if the transition is invalid
    fn compute_transition(
        &self,
        current: RecordingState,
        event: RecordingEvent,
    ) -> Option<(RecordingState, Option<RecordingAction>)> {
        match current {
            RecordingState::Ready => match event {
                RecordingEvent::FnDown => Some((
                    RecordingState::Recording,
                    Some(RecordingAction::StartRecording),
                )),
                RecordingEvent::Retry => Some((
                    RecordingState::Transcribing,
                    Some(RecordingAction::RetryTranscription),
                )),
                _ => None,
            },

            RecordingState::Recording => match event {
                RecordingEvent::FnUp => Some((
                    RecordingState::Transcribing,
                    Some(RecordingAction::StopAndTranscribe),
                )),
                RecordingEvent::Lock => Some((RecordingState::RecordingLocked, None)),
                RecordingEvent::Cancel => Some((
                    RecordingState::Ready,
                    Some(RecordingAction::CancelRecording),
                )),
                _ => None,
            },

            RecordingState::RecordingLocked => match event {
                RecordingEvent::FnDown => Some((
                    RecordingState::Transcribing,
                    Some(RecordingAction::StopAndTranscribe),
                )),
                RecordingEvent::Cancel => Some((
                    RecordingState::Ready,
                    Some(RecordingAction::CancelRecording),
                )),
                _ => None,
            },

            RecordingState::Transcribing => match event {
                RecordingEvent::TranscriptionComplete | RecordingEvent::TranscriptionFailed => {
                    Some((RecordingState::Ready, None))
                }
                _ => None,
            },
        }
    }

    /// Force reset to Ready state (for error recovery)
    /// Use sparingly - prefer transition() for normal flow
    pub fn reset(&self) {
        *self.state.lock().unwrap() = RecordingState::Ready;
    }
}

impl Default for RecordingStateManager {
    fn default() -> Self {
        Self::new()
    }
}
