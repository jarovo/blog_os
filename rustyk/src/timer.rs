use futures_util::Stream;
use nostd::time::Duration;
use crate::task::{reactor::Reactor};
use crate::clock::Instant;
use core::task::{Context, Poll};

use core::task::Waker;


use core::pin::Pin;

// https://docs.rs/async-io/latest/src/async_io/lib.rs.html#144-159


pub fn timer_after(dur: Duration) -> Timer {
    // Implementation of timer creation that will be ready after the specified number of ticks
    // TODO: Handle overflow of Instant when adding duration
    Timer { id_and_waker: None, when: Some(Instant::now().checked_add(dur).unwrap() ), period: Duration::MAX }
}

#[derive(Debug)]
pub struct Timer {
    /// This timer's ID and last waker that polled it.
    ///
    /// When this field is set to `None`, this timer is not registered in the reactor.
    id_and_waker: Option<(usize, Waker)>,

    /// The next instant at which this timer fires.
    ///
    /// If this timer is a blank timer, this value is None. If the timer
    /// must be set, this value contains the next instant at which the
    /// timer must fire.
    when: Option<Instant>,

    /// The period.
    period: Duration,
}

impl Timer {
    /// Creates a timer that will never fire.
    ///
    /// # Examples
    ///
    /// This function may also be useful for creating a function with an optional timeout.
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_io::Timer;
    /// use futures_lite::prelude::*;
    /// use std::time::Duration;
    ///
    /// async fn run_with_timeout(timeout: Option<Duration>) {
    ///     let timer = timeout
    ///         .map(|timeout| Timer::after(timeout))
    ///         .unwrap_or_else(Timer::never);
    ///
    ///     run_lengthy_operation().or(timer).await;
    /// }
    /// # // Note that since a Timer as a Future returns an Instant,
    /// # // this function needs to return an Instant to be used
    /// # // in "or".
    /// # async fn run_lengthy_operation() -> std::time::Instant {
    /// #    std::time::Instant::now()
    /// # }
    ///
    /// // Times out after 5 seconds.
    /// run_with_timeout(Some(Duration::from_secs(5))).await;
    /// // Does not time out.
    /// run_with_timeout(None).await;
    /// # });
    /// ```
    pub fn never() -> Timer {
        Timer {
            id_and_waker: None,
            when: None,
            period: Duration::MAX,
        }
    }

    /// Creates a timer that emits an event once after the given duration of time.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use std::time::Duration;
    ///
    /// # futures_lite::future::block_on(async {
    /// Timer::after(Duration::from_secs(1)).await;
    /// # });
    /// ```
    pub fn after(duration: Duration) -> Timer {
        Instant::now()
            .checked_add(duration)
            .map_or_else(Timer::never, Timer::at)
    }

    /// Creates a timer that emits an event once at the given time instant.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let now = Instant::now();
    /// let when = now + Duration::from_secs(1);
    /// Timer::at(when).await;
    /// # });
    /// ```
    pub fn at(instant: Instant) -> Timer {
        Timer::interval_at(instant, Duration::MAX)
    }

    /// Creates a timer that emits events periodically.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use futures_lite::StreamExt;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let period = Duration::from_secs(1);
    /// Timer::interval(period).next().await;
    /// # });
    /// ```
    pub fn interval(period: Duration) -> Timer {
        Instant::now()
            .checked_add(period)
            .map_or_else(Timer::never, |at| Timer::interval_at(at, period))
    }

    /// Creates a timer that emits events periodically, starting at `start`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use futures_lite::StreamExt;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let start = Instant::now();
    /// let period = Duration::from_secs(1);
    /// Timer::interval_at(start, period).next().await;
    /// # });
    /// ```
    pub fn interval_at(start: Instant, period: Duration) -> Timer {
        Timer {
            id_and_waker: None,
            when: Some(start),
            period,
        }
    }

    /// Indicates whether or not this timer will ever fire.
    ///
    /// [`never()`] will never fire, and timers created with [`after()`] or [`at()`] will fire
    /// if the duration is not too large.
    ///
    /// [`never()`]: Timer::never()
    /// [`after()`]: Timer::after()
    /// [`at()`]: Timer::at()
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_io::Timer;
    /// use futures_lite::prelude::*;
    /// use std::time::Duration;
    ///
    /// // `never` will never fire.
    /// assert!(!Timer::never().will_fire());
    ///
    /// // `after` will fire if the duration is not too large.
    /// assert!(Timer::after(Duration::from_secs(1)).will_fire());
    /// assert!(!Timer::after(Duration::MAX).will_fire());
    ///
    /// // However, once an `after` timer has fired, it will never fire again.
    /// let mut t = Timer::after(Duration::from_secs(1));
    /// assert!(t.will_fire());
    /// (&mut t).await;
    /// assert!(!t.will_fire());
    ///
    /// // Interval timers will fire periodically.
    /// let mut t = Timer::interval(Duration::from_secs(1));
    /// assert!(t.will_fire());
    /// t.next().await;
    /// assert!(t.will_fire());
    /// # });
    /// ```
    #[inline]
    pub fn will_fire(&self) -> bool {
        self.when.is_some()
    }

    /// Sets the timer to emit an event once after the given duration of time.
    ///
    /// Note that resetting a timer is different from creating a new timer because
    /// [`set_after()`][`Timer::set_after()`] does not remove the waker associated with the task
    /// that is polling the timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use std::time::Duration;
    ///
    /// # futures_lite::future::block_on(async {
    /// let mut t = Timer::after(Duration::from_secs(1));
    /// t.set_after(Duration::from_millis(100));
    /// # });
    /// ```
    pub fn set_after(&mut self, duration: Duration) {
        match Instant::now().checked_add(duration) {
            Some(instant) => self.set_at(instant),
            None => {
                // Overflow to never going off.
                self.clear();
            }
        }
    }

    /// Sets the timer to emit an event once at the given time instant.
    ///
    /// Note that resetting a timer is different from creating a new timer because
    /// [`set_at()`][`Timer::set_at()`] does not remove the waker associated with the task
    /// that is polling the timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let mut t = Timer::after(Duration::from_secs(1));
    ///
    /// let now = Instant::now();
    /// let when = now + Duration::from_secs(1);
    /// t.set_at(when);
    /// # });
    /// ```
    pub fn set_at(&mut self, instant: Instant) {
        self.clear();

        // Update the timeout.
        self.when = Some(instant);

        if let Some((id, waker)) = self.id_and_waker.as_mut() {
            // Re-register the timer with the new timeout.
            *id = Reactor::get().insert_timer(instant, waker);
        }
    }

    /// Sets the timer to emit events periodically.
    ///
    /// Note that resetting a timer is different from creating a new timer because
    /// [`set_interval()`][`Timer::set_interval()`] does not remove the waker associated with the
    /// task that is polling the timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use futures_lite::StreamExt;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let mut t = Timer::after(Duration::from_secs(1));
    ///
    /// let period = Duration::from_secs(2);
    /// t.set_interval(period);
    /// # });
    /// ```
    pub fn set_interval(&mut self, period: Duration) {
        match Instant::now().checked_add(period) {
            Some(instant) => self.set_interval_at(instant, period),
            None => {
                // Overflow to never going off.
                self.clear();
            }
        }
    }

    /// Sets the timer to emit events periodically, starting at `start`.
    ///
    /// Note that resetting a timer is different from creating a new timer because
    /// [`set_interval_at()`][`Timer::set_interval_at()`] does not remove the waker associated with
    /// the task that is polling the timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_io::Timer;
    /// use futures_lite::StreamExt;
    /// use std::time::{Duration, Instant};
    ///
    /// # futures_lite::future::block_on(async {
    /// let mut t = Timer::after(Duration::from_secs(1));
    ///
    /// let start = Instant::now();
    /// let period = Duration::from_secs(2);
    /// t.set_interval_at(start, period);
    /// # });
    /// ```
    pub fn set_interval_at(&mut self, start: Instant, period: Duration) {
        self.clear();

        self.when = Some(start);
        self.period = period;

        if let Some((id, waker)) = self.id_and_waker.as_mut() {
            // Re-register the timer with the new timeout.
            *id = Reactor::get().insert_timer(start, waker);
        }
    }

    /// Clear any timeouts set on this timer. It will never fire again until a new interval or instant is set.
    pub fn clear(&mut self) {
        if let (Some(when), Some((id, _))) = (self.when, self.id_and_waker.as_ref()) {
            // Deregister the timer from the reactor.
            Reactor::get().remove_timer(when, *id);
        }
        self.when = None;
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let (Some(when), Some((id, _))) = (self.when, self.id_and_waker.take()) {
            // Deregister the timer from the reactor.
            Reactor::get().remove_timer(when, id);
        }
    }
}

impl Future for Timer {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(when)) => Poll::Ready(when),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => unreachable!(),
        }
    }
}

impl Stream for Timer {
    type Item = Instant;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(ref mut when) = this.when {
            // Check if the timer has already fired.
            if Instant::now() >= *when {
                if let Some((id, _)) = this.id_and_waker.take() {
                    // Deregister the timer from the reactor.
                    Reactor::get().remove_timer(*when, id);
                }
                let result_time = *when;
                if let Some(next) = (*when).checked_add(this.period) {
                    *when = next;
                    // Register the timer in the reactor.
                    let id = Reactor::get().insert_timer(next, cx.waker());
                    this.id_and_waker = Some((id, cx.waker().clone()));
                } else {
                    this.when = None;
                }
                return Poll::Ready(Some(result_time));
            } else {
                match &this.id_and_waker {
                    None => {
                        // Register the timer in the reactor.
                        let id = Reactor::get().insert_timer(*when, cx.waker());
                        this.id_and_waker = Some((id, cx.waker().clone()));
                    }
                    Some((id, w)) if !w.will_wake(cx.waker()) => {
                        // Deregister the timer from the reactor to remove the old waker.
                        Reactor::get().remove_timer(*when, *id);

                        // Register the timer in the reactor with the new waker.
                        let id = Reactor::get().insert_timer(*when, cx.waker());
                        this.id_and_waker = Some((id, cx.waker().clone()));
                    }
                    Some(_) => {}
                }
            }
        }

        Poll::Pending
    }
}