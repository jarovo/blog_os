use lazy_static::lazy_static;
use core::sync::atomic::{AtomicU64, Ordering};


lazy_static! {
    static ref TICKS: AtomicU64 = AtomicU64::new(0);
}

pub struct Clock;

impl Clock {
    pub fn new() -> Self {
        Clock
    }

    pub fn tick(&self) {
        TICKS.fetch_add(1, Ordering::SeqCst);
    }

    pub fn ticks(&self) -> u64 {
        TICKS.load(Ordering::SeqCst)
    }
}