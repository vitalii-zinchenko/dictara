use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

// Embed sound files at compile time
const START_SOUND: &[u8] = include_bytes!("../sounds/start.wav");
const STOP_SOUND: &[u8] = include_bytes!("../sounds/stop.wav");

/// Play the start sound (non-blocking) at 50% volume
pub fn play_start() {
    std::thread::spawn(|| {
        if let Err(e) = play_sound(START_SOUND, 0.5) {
            eprintln!("[SoundPlayer] Failed to play start sound: {}", e);
        }
    });
}

/// Play the stop sound (non-blocking)
pub fn play_stop() {
    std::thread::spawn(|| {
        if let Err(e) = play_sound(STOP_SOUND, 1.0) {
            eprintln!("[SoundPlayer] Failed to play stop sound: {}", e);
        }
    });
}

fn play_sound(sound_data: &'static [u8], volume: f32) -> Result<(), String> {
    let (_stream, stream_handle) =
        OutputStream::try_default().map_err(|e| format!("Failed to get output stream: {}", e))?;

    let cursor = Cursor::new(sound_data);
    let source = Decoder::new(cursor).map_err(|e| format!("Failed to decode sound: {}", e))?;

    let sink =
        Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;

    sink.set_volume(volume);
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
