use core::ops::Add;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicU64, Ordering};
use nostd::time::Duration;

use crate::task::timeout::timeout;
use crate::task::future;
use anyhow::Result;

lazy_static! {
    static ref TICKS: AtomicU64 = AtomicU64::new(0);
}

pub(crate) fn tick() {
    TICKS.fetch_add(1, Ordering::SeqCst);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    pub fn now() -> Instant {
        Instant { ticks: TICKS.load(Ordering::SeqCst) }
    }

    pub fn checked_add(&self, dur: Duration) -> Option<Instant> {
        self.ticks.checked_add(dur.as_millis() as u64).map(|ticks| Instant { ticks })
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        if self.ticks >= earlier.ticks {
            Duration::from_millis((self.ticks - earlier.ticks) as u64)
        } else {
            Duration::from_millis(0)
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, dur: Duration) -> Instant {
        self.checked_add(dur).expect("Instant overflow")
    }
}

pub struct Ticker {
    period: Duration,
    last_scheduled_tick: Instant,
}

impl Ticker {
    pub fn new(period: Duration) -> Self {
        Ticker {
            period,
            last_scheduled_tick: Instant::now(),
        }
    }

    pub async fn tick(&mut self) {
        self.last_scheduled_tick = self.last_scheduled_tick.checked_add(self.period).unwrap();
        sleep(Duration::from_millis(self.last_scheduled_tick.ticks - Instant::now().ticks)).await;
    }
}

pub async fn sleep(dur: Duration) {
    let _: Result<()> = timeout(dur,  future::pending()).await;
}