use crate::recording::RecordingCommand;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRunLoop};
#[cfg(target_os = "macos")]
use objc2_core_graphics::{
    kCGEventMaskForAllEvents, CGEvent, CGEventField, CGEventTapCallBack, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
#[cfg(target_os = "macos")]
use std::{ffi::c_void, ptr::NonNull};

#[cfg(any(not(target_os = "macos")))]
use rdev::{listen, Event, EventType, Key, ListenError};
#[cfg(target_os = "macos")]
use rdev::{listen, EventType, Key};

/// Stateful FN key listener
pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
}

impl KeyListener {
    pub fn start(
        command_tx: mpsc::Sender<RecordingCommand>,
        recording_state: Arc<AtomicU8>,
    ) -> Self {
        #[cfg(target_os = "macos")]
        {
            return Self::start_macos(command_tx, recording_state);
        }

        #[cfg(not(target_os = "macos"))]
        {
            return Self::start_rdev(command_tx);
        }
    }

    #[cfg(target_os = "macos")]
    fn start_macos(
        command_tx: mpsc::Sender<RecordingCommand>,
        recording_state: Arc<AtomicU8>,
    ) -> Self {
        let thread_handle = thread::spawn(move || {
            println!("[FN Key Listener] Starting CGEvent tap listener...");

            if let Err(err) = run_event_tap(command_tx.clone(), recording_state.clone()) {
                eprintln!(
                    "[FN Key Listener] CGEvent tap failed: {}. Falling back to rdev::listen (emoji picker may appear).",
                    err
                );
                if let Err(listen_err) = listen(move |event: rdev::Event| match event.event_type {
                    EventType::KeyPress(Key::Function) => {
                        let _ = command_tx.blocking_send(RecordingCommand::FnDown);
                    }
                    EventType::KeyRelease(Key::Function) => {
                        let _ = command_tx.blocking_send(RecordingCommand::FnUp);
                    }
                    EventType::KeyPress(Key::Space) => {
                        let _ = command_tx.blocking_send(RecordingCommand::Lock);
                    }
                    _ => {}
                }) {
                    eprintln!(
                        "[FN Key Listener] rdev::listen fallback failed: {:?}",
                        listen_err
                    );
                }
            }
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn start_rdev(command_tx: mpsc::Sender<RecordingCommand>) -> Self {
        let thread_handle = thread::spawn(move || {
            println!("[FN Key Listener] Starting global keyboard listener...");

            let listen_res = listen(move |event: Event| match event.event_type {
                EventType::KeyPress(Key::Function) => {
                    let _ = command_tx.blocking_send(RecordingCommand::FnDown);
                }
                EventType::KeyRelease(Key::Function) => {
                    let _ = command_tx.blocking_send(RecordingCommand::FnUp);
                }
                EventType::KeyPress(Key::Space) => {
                    let _ = command_tx.blocking_send(RecordingCommand::Lock);
                }
                _ => {}
            });

            if let Err(error) = listen_res {
                let error_msg = match error {
                    ListenError::EventTapError => {
                        "macOS Accessibility permission denied. Please grant permission and restart."
                    }
                    _ => "Keyboard listener failed",
                };
                eprintln!("[FN Key Listener] {} ({:?})", error_msg, error);
            }

            println!("[FN Key Listener] Thread exiting");
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }
}

#[cfg(target_os = "macos")]
struct CallbackState {
    command_tx: mpsc::Sender<RecordingCommand>,
    recording_state: Arc<AtomicU8>,
    fn_down: bool,
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    // Key codes from <HIToolbox/Events.h>
    const KEYCODE_FN: i64 = 63;
    const KEYCODE_SPACE: i64 = 49;

    let state = &mut *(user_info as *mut CallbackState);

    let keycode =
        CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField::KeyboardEventKeycode);

    match event_type {
        CGEventType::KeyDown => {
            if keycode == KEYCODE_FN {
                state.fn_down = true;
                let _ = state.command_tx.blocking_send(RecordingCommand::FnDown);
                return std::ptr::null_mut(); // Swallow to block emoji picker
            } else if keycode == KEYCODE_SPACE {
                let current_state = state.recording_state.load(Ordering::Relaxed);
                if current_state == 1 {
                    // Only use Space to lock while actively recording; pass through otherwise
                    let _ = state.command_tx.blocking_send(RecordingCommand::Lock);
                    return std::ptr::null_mut(); // Avoid inserting a space while recording
                }
            }
        }
        CGEventType::KeyUp => {
            if keycode == KEYCODE_FN {
                state.fn_down = false;
                let _ = state.command_tx.blocking_send(RecordingCommand::FnUp);
                return std::ptr::null_mut(); // Swallow to block emoji picker
            }
        }
        CGEventType::FlagsChanged => {
            if keycode == KEYCODE_FN {
                // Fn often arrives as FlagsChanged events; toggle based on last state
                if state.fn_down {
                    state.fn_down = false;
                    let _ = state.command_tx.blocking_send(RecordingCommand::FnUp);
                } else {
                    state.fn_down = true;
                    let _ = state.command_tx.blocking_send(RecordingCommand::FnDown);
                }
                return std::ptr::null_mut();
            }
        }
        _ => {}
    }

    cg_event.as_ptr()
}

#[cfg(target_os = "macos")]
fn run_event_tap(
    command_tx: mpsc::Sender<RecordingCommand>,
    recording_state: Arc<AtomicU8>,
) -> Result<(), String> {
    unsafe {
        let callback_state = Box::new(CallbackState {
            command_tx,
            recording_state,
            fn_down: false,
        });
        let user_info = Box::into_raw(callback_state) as *mut c_void;
        let callback: CGEventTapCallBack = Some(tap_callback);

        let tap = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            kCGEventMaskForAllEvents.into(),
            callback,
            user_info,
        )
        .ok_or_else(|| "Failed to create CGEvent tap (accessibility permission?)".to_string())?;

        let loop_source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or_else(|| "Failed to create run loop source for event tap".to_string())?;

        let current_loop =
            CFRunLoop::current().ok_or_else(|| "Failed to get current CFRunLoop".to_string())?;

        current_loop.add_source(Some(&loop_source), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);

        // This blocks the thread until the run loop is stopped
        CFRunLoop::run();

        // If the loop ever exits, reclaim the boxed state
        let _ = Box::from_raw(user_info as *mut CallbackState);
    }

    Ok(())
}
