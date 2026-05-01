use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use alloc::boxed::Box;

use crate::clock::Clock;

pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

struct SleepFuture {
    scheduled_ticks: u64,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if Clock.ticks() >= self.scheduled_ticks {
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