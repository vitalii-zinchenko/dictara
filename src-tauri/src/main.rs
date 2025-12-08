// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    typefree_lib::run()
}

// use std::thread;

// use rdev::{listen, Event};

// fn main() {
//     // This will block.
//     // if let Err(error) = listen(callback) {
//     //     println!("Error: {:?}", error)
//     // }

//     let _listener = thread::spawn(move || {
//         listen(move |event| {
//             println!("Event: {:?}", event.event_type);
//         })
//         .expect("Could not listen");
//     });

//     println!("End 1")

//     if let Err(error) = listen(callback) {
//         println!("Error: {:?}", error)
//     }

//     println!("End")
// }

// fn callback(event: Event) {
//     println!("My callback {:?}", event);
// }
