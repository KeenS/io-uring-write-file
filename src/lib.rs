use std::time::{Duration, Instant};

pub const TOTAL: usize = 5 * 1024 * 1024 * 1024;
pub static DATA: [u8; 4096] = [0x61; 4096];

#[derive(Debug)]
pub struct Timer(Instant);

impl Timer {
    pub fn start() -> Self {
        Timer(Instant::now())
    }

    pub fn stop(self) -> Duration {
        let now = Instant::now();
        now - self.0
    }
}
