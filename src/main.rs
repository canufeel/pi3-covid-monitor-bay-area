use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::thread;

mod draw;
mod request;
mod stop;
mod payload;

use std::sync::atomic::{Ordering};
use crate::stop::STOP;

use crate::payload::{PayloadStorage, handle_data_updates};
use std::sync::{Arc, RwLock};

pub fn main() -> Result<(), String> {
    let (tx, rx) = channel();
    let request_handle = thread::spawn(move || {
        match request::get_data_loop(tx) {
            Ok(_) => {},
            Err(x) => println!("{}", x)
        }
    });

    let payload_storage = Arc::new(RwLock::new(PayloadStorage::new()));
    let storage_copy = payload_storage.clone();

    let update_handle = thread::spawn(move || {
        handle_data_updates(rx, storage_copy);
    });

    let mut ctx = draw::Context::new()?;
    ctx.draw(payload_storage.clone())?;
    let mut event_pump = ctx.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    STOP.store(true, Ordering::Release);
                    request_handle.join().unwrap();
                    update_handle.join().unwrap();
                    break 'running
                },
                _ => {}
            }
        }

        ctx.draw(payload_storage.clone())?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
