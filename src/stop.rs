use std::sync::atomic::{AtomicBool};

pub static STOP: AtomicBool = AtomicBool::new(false);
