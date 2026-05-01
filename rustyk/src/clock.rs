use lazy_static::lazy_static;
use core::sync::atomic::{AtomicU64, Ordering};
use futures_util::task::AtomicWaker;
use futures_util::Future;
use core::task::{Context, Poll};
use core::pin::Pin;

static WAKER: AtomicWaker = AtomicWaker::new();


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
        WAKER.wake();
    }

    pub fn ticks(&self) -> u64 {
        TICKS.load(Ordering::SeqCst)
    }
}


struct SleepFuture {
    scheduled_ticks: u64,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        WAKER.register(&cx.waker());
        if Clock.ticks() >= self.scheduled_ticks {
            WAKER.take();
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub fn sleep_clock_ticks(ticks: u64) -> impl Future<Output = ()> {
    SleepFuture {
        scheduled_ticks: Clock.ticks() + ticks,
    }
}

pub struct Ticker {
    period: u64,
    last_scheduled_tick: u64,
}

impl Ticker {
    pub fn new(period: u64) -> Self {
        Ticker {
            period,
            last_scheduled_tick: Clock.ticks(),
        }
    }

    pub fn tick(&mut self) -> impl Future<Output = ()> {
        self.last_scheduled_tick += self.period;
        SleepFuture {
            scheduled_ticks: self.last_scheduled_tick,
        }
    }
}