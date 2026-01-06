//! macOS implementation using CGEvent tap.

use crate::{Event, EventType, GrabError, Key};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRunLoop};
use objc2_core_graphics::{
    kCGEventMaskForAllEvents, CGEvent, CGEventField, CGEventTapCallBack, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use std::ffi::c_void;
use std::ptr::NonNull;

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

/// Start grabbing keyboard events using CGEvent tap.
///
/// This function blocks the current thread.
pub fn grab<F>(callback: F) -> Result<(), GrabError>
where
    F: FnMut(Event) -> Option<Event> + 'static,
{
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

        let loop_source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or(GrabError::LoopSourceError)?;

        let current_loop = CFRunLoop::current().ok_or(GrabError::RunLoopError)?;

        current_loop.add_source(Some(&loop_source), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);

        // This blocks until the run loop is stopped
        CFRunLoop::run();

        // Cleanup if the loop ever exits
        let _ = Box::from_raw(user_info as *mut CallbackState);
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
