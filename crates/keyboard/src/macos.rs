//! macOS implementation using CGEvent tap.

use crate::{Event, EventType, GrabError, Key};
use log::{error, info, warn};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRunLoop};
use objc2_core_graphics::{
    kCGEventMaskForAllEvents, CGEvent, CGEventField, CGEventTapCallBack, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// How often to check if accessibility permission is still granted.
const ACCESSIBILITY_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Global reference to the event tap for re-enabling after timeout.
/// This is safe because we only have one tap per process.
static TAP_REF: AtomicPtr<CFMachPort> = AtomicPtr::new(std::ptr::null_mut());

/// Global reference to the run loop for stopping from the polling thread.
///
/// # Safety invariants
///
/// This pointer is valid as long as the main thread is blocked on `CFRunLoop::run()`.
/// The polling thread may only dereference this pointer while it's running, and the
/// main thread only clears this pointer AFTER joining the polling thread.
static RUN_LOOP_REF: AtomicPtr<CFRunLoop> = AtomicPtr::new(std::ptr::null_mut());

/// State passed to the CGEvent callback.
struct CallbackState {
    callback: Box<dyn FnMut(Event) -> Option<Event>>,
    /// Track modifier key states for FlagsChanged events
    fn_down: bool,
    control_left_down: bool,
    control_right_down: bool,
    alt_down: bool,
    alt_gr_down: bool,
    meta_left_down: bool,
    meta_right_down: bool,
}

/// Check if accessibility permission is currently granted.
fn check_accessibility() -> bool {
    // Use the macos-accessibility-client crate's function
    macos_accessibility_client::accessibility::application_is_trusted()
}

/// Start grabbing keyboard events using CGEvent tap.
///
/// This function blocks the current thread.
pub fn grab<F>(callback: F) -> Result<(), GrabError>
where
    F: FnMut(Event) -> Option<Event> + 'static,
{
    // Check accessibility permission upfront for a clear error
    if !check_accessibility() {
        return Err(GrabError::AccessibilityNotGranted);
    }

    unsafe {
        let state = Box::new(CallbackState {
            callback: Box::new(callback),
            fn_down: false,
            control_left_down: false,
            control_right_down: false,
            alt_down: false,
            alt_gr_down: false,
            meta_left_down: false,
            meta_right_down: false,
        });
        let user_info = Box::into_raw(state) as *mut c_void;

        let tap_callback: CGEventTapCallBack = Some(event_tap_callback);

        let tap = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            kCGEventMaskForAllEvents.into(),
            tap_callback,
            user_info,
        )
        .ok_or(GrabError::EventTapError)?;

        // Store tap reference globally so callback can re-enable it
        // We need to store the raw pointer since CFMachPort isn't Send
        // Use as_ref() to get the inner reference, then cast to raw pointer
        let tap_ptr = (&*tap) as *const CFMachPort as *mut CFMachPort;
        TAP_REF.store(tap_ptr, Ordering::SeqCst);

        let loop_source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or(GrabError::LoopSourceError)?;

        let current_loop = CFRunLoop::current().ok_or(GrabError::RunLoopError)?;

        current_loop.add_source(Some(&loop_source), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);
        info!("Event tap started successfully");

        // Store run loop reference so the polling thread can stop it
        let run_loop_ptr = (&*current_loop) as *const CFRunLoop as *mut CFRunLoop;
        RUN_LOOP_REF.store(run_loop_ptr, Ordering::SeqCst);

        // Start accessibility polling thread
        // This thread checks every 200ms if accessibility permission is still granted
        // If permission is revoked, it stops the run loop to prevent system freeze
        let stop_polling = Arc::new(AtomicBool::new(false));
        let stop_polling_clone = Arc::clone(&stop_polling);

        let polling_thread = thread::Builder::new()
            .name("accessibility-poll".to_string())
            .spawn(move || {
                info!("Accessibility polling thread started");
                while !stop_polling_clone.load(Ordering::SeqCst) {
                    thread::sleep(ACCESSIBILITY_POLL_INTERVAL);

                    if !check_accessibility() {
                        error!("Accessibility permission lost (detected by polling), stopping event tap");
                        let rl_ptr = RUN_LOOP_REF.load(Ordering::SeqCst);
                        if !rl_ptr.is_null() {
                            // SAFETY: The run loop pointer is valid because:
                            // 1. The main thread is blocked on CFRunLoop::run()
                            // 2. Cleanup only happens AFTER this thread is joined
                            // CFRunLoop::stop is thread-safe
                            (*rl_ptr).stop();
                        }
                        break;
                    }
                }
                info!("Accessibility polling thread stopped");
            })
            .expect("Failed to spawn accessibility polling thread");

        // This blocks until the run loop is stopped
        CFRunLoop::run();

        // Signal the polling thread to stop and wait for it
        stop_polling.store(true, Ordering::SeqCst);
        let _ = polling_thread.join();

        // Cleanup - safe to clear now since polling thread has been joined
        RUN_LOOP_REF.store(std::ptr::null_mut(), Ordering::SeqCst);
        TAP_REF.store(std::ptr::null_mut(), Ordering::SeqCst);
        let _ = Box::from_raw(user_info as *mut CallbackState);
        info!("Event tap stopped");
    }

    Ok(())
}

/// The CGEvent tap callback.
///
/// # Safety
///
/// This is called from the system's event tap. The `user_info` must be a valid
/// pointer to a `CallbackState` that was created with `Box::into_raw`.
unsafe extern "C-unwind" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    // Handle tap disabled events first - these are critical for stability
    match event_type {
        CGEventType::TapDisabledByTimeout => {
            warn!("Event tap disabled by timeout, checking accessibility...");
            // Check if we still have accessibility permission before re-enabling
            if check_accessibility() {
                let tap_ptr = TAP_REF.load(Ordering::SeqCst);
                if !tap_ptr.is_null() {
                    CGEvent::tap_enable(&*tap_ptr, true);
                    info!("Event tap re-enabled after timeout");
                }
            } else {
                // No accessibility permission - stop gracefully instead of re-enabling
                error!("Accessibility permission lost, stopping event tap");
                if let Some(rl) = CFRunLoop::current() {
                    rl.stop();
                }
            }
            return cg_event.as_ptr();
        }
        CGEventType::TapDisabledByUserInput => {
            warn!("Event tap disabled by user input, checking accessibility...");
            // Check if accessibility permission is still granted
            if check_accessibility() {
                // Still have permission, re-enable the tap
                let tap_ptr = TAP_REF.load(Ordering::SeqCst);
                if !tap_ptr.is_null() {
                    CGEvent::tap_enable(&*tap_ptr, true);
                    info!("Event tap re-enabled after user input check");
                }
            } else {
                // Accessibility was revoked - stop the run loop gracefully
                error!("Accessibility permission revoked, stopping event tap");
                if let Some(rl) = CFRunLoop::current() {
                    rl.stop();
                }
            }
            return cg_event.as_ptr();
        }
        _ => {}
    }

    let state = &mut *(user_info as *mut CallbackState);

    // Get the keycode
    let keycode =
        CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField::KeyboardEventKeycode);
    let key = Key::from_macos_keycode(keycode as u32);

    // Convert to our event type
    let event = match event_type {
        CGEventType::KeyDown => Some(Event::new(EventType::KeyPress(key))),
        CGEventType::KeyUp => Some(Event::new(EventType::KeyRelease(key))),
        CGEventType::FlagsChanged => {
            // For modifier keys (including Fn), FlagsChanged is sent instead of KeyDown/KeyUp.
            // We track state to determine if it's a press or release.
            let (is_down, set_down): (bool, &mut bool) = match key {
                Key::Function => (state.fn_down, &mut state.fn_down),
                Key::ControlLeft => (state.control_left_down, &mut state.control_left_down),
                Key::ControlRight => (state.control_right_down, &mut state.control_right_down),
                Key::Alt => (state.alt_down, &mut state.alt_down),
                Key::AltGr => (state.alt_gr_down, &mut state.alt_gr_down),
                Key::MetaLeft => (state.meta_left_down, &mut state.meta_left_down),
                Key::MetaRight => (state.meta_right_down, &mut state.meta_right_down),
                _ => {
                    // For other modifiers (Shift, CapsLock, etc.), just emit as press
                    return cg_event.as_ptr();
                }
            };

            if is_down {
                *set_down = false;
                Some(Event::new(EventType::KeyRelease(key)))
            } else {
                *set_down = true;
                Some(Event::new(EventType::KeyPress(key)))
            }
        }
        _ => None,
    };

    // If we got a keyboard event, call the user's callback
    if let Some(event) = event {
        let result = (state.callback)(event);
        if result.is_none() {
            // User wants to swallow this event
            return std::ptr::null_mut();
        }
    }

    // Pass the event through
    cg_event.as_ptr()
}
