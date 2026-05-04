use alloc::{collections::BTreeMap};
use anyhow::Result;
use spin::mutex::{Mutex, MutexGuard};
use core::time::Duration;
use crate::clock::{Instant};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::Waker;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use concurrent_queue::ConcurrentQueue;
use core::mem;

#[cfg(not(target_os = "espidf"))]
const TIMER_QUEUE_SIZE: usize = 1000;

/// ESP-IDF - being an embedded OS - does not need so many timers
/// and this saves ~ 20K RAM which is a lot for an MCU with RAM < 400K
#[cfg(target_os = "espidf")]
const TIMER_QUEUE_SIZE: usize = 100;

struct Events {
    // In a real implementation, this would be a list of events returned by the OS.
    // For this example, we just use an empty struct.
}

impl Events {
    fn clear(&mut self) {
        // In a real implementation, this would clear the list of events.
        // For this example, we just do nothing.
    }
}

pub struct Reactor { 
    ticker: AtomicUsize,
    events: Mutex<Events>,
    timers: Mutex<BTreeMap<(Instant, usize), Waker>>,
    timer_ops: ConcurrentQueue<TimerOp>,
}

impl Reactor {

    pub fn get() -> &'static Reactor {
        static REACTOR: OnceCell<Reactor> = OnceCell::uninit();

        let reactor = REACTOR.get_or_init(|| Reactor {
            ticker: AtomicUsize::new(0),
            events: Mutex::new(Events {}),
            timers: Mutex::new(BTreeMap::new()),
            timer_ops: ConcurrentQueue::bounded(TIMER_QUEUE_SIZE),
        });
        reactor
    }

    /// Registers a timer in the reactor.
    ///
    /// Returns the inserted timer's ID.
    pub(crate) fn insert_timer(&self, when: Instant, waker: &Waker) -> usize {
        // Generate a new timer ID.
        static ID_GENERATOR: AtomicUsize = AtomicUsize::new(1);
        let id = ID_GENERATOR.fetch_add(1, Ordering::Relaxed);

        // Push an insert operation.
        while self
            .timer_ops
            .push(TimerOp::Insert(when, id, waker.clone()))
            .is_err()
        {
            // If the queue is full, drain it and try again.
            let mut timers = self.timers.lock();
            self.process_timer_ops(&mut timers);
        }

        // Notify that a timer has been inserted.
        self.notify();

        id
    }


    /// Deregisters a timer from the reactor.
    pub(crate) fn remove_timer(&self, when: Instant, id: usize) {
        // Push a remove operation.
        while self.timer_ops.push(TimerOp::Remove(when, id)).is_err() {
            // If the queue is full, drain it and try again.
            let mut timers = self.timers.lock();
            self.process_timer_ops(&mut timers);
        }
    }

    /// Processes queued timer operations.
    fn process_timer_ops(&self, timers: &mut MutexGuard<'_, BTreeMap<(Instant, usize), Waker>>) {
        // Process only as much as fits into the queue, or else this loop could in theory run
        // forever.
        self.timer_ops
            .try_iter()
            .take(self.timer_ops.capacity().unwrap())
            .for_each(|op| match op {
                TimerOp::Insert(when, id, waker) => {
                    timers.insert((when, id), waker);
                }
                TimerOp::Remove(when, id) => {
                    timers.remove(&(when, id));
                }
            });
    }
    
    /// Notifies the thread blocked on the reactor.
    pub(crate) fn notify(&self) {
        // In a real implementation, this would trigger an interrupt or something similar to wake
        // the thread blocked on the reactor. For this example, we just do nothing.

        // self.poller.notify().expect("failed to notify reactor");
    }


    /// Locks the reactor, potentially blocking if the lock is held by another thread.
    pub(crate) fn lock(&self) -> ReactorLock<'_> {
        let reactor = self;
        let events = self.events.lock();
        ReactorLock { reactor, events }
    }

    /// Attempts to lock the reactor.
    pub(crate) fn try_lock(&self) -> Option<ReactorLock<'_>> {
        self.events.try_lock().map(|events| {
            let reactor = self;
            ReactorLock { reactor, events }
        })
    }

    /// Processes ready timers and extends the list of wakers to wake.
    ///
    /// Returns the duration until the next timer before this method was called.
    fn process_timers(&self, wakers: &mut Vec<Waker>) -> Option<Duration> {
        #[cfg(feature = "tracing")]
        let span = tracing::trace_span!("process_timers");
        #[cfg(feature = "tracing")]
        let _enter = span.enter();

        let mut timers = self.timers.lock();
        self.process_timer_ops(&mut timers);

        let now = Instant::now();

        // Split timers into ready and pending timers.
        //
        // Careful to split just *after* `now`, so that a timer set for exactly `now` is considered
        // ready.
        let pending = timers.split_off(&(now + Duration::from_nanos(1), 0));
        let ready = mem::replace(&mut *timers, pending);

        // Calculate the duration until the next event.
        let dur = if ready.is_empty() {
            // Duration until the next timer.
            timers
                .keys()
                .next()
                .map(|(when, _)| when.saturating_duration_since(now))
        } else {
            // Timers are about to fire right now.
            Some(Duration::from_secs(0))
        };

        // Drop the lock before waking.
        drop(timers);

        // Add wakers to the list.
        #[cfg(feature = "tracing")]
        tracing::trace!("{} ready wakers", ready.len());

        for (_, waker) in ready {
            wakers.push(waker);
        }

        dur
    }
}


/// A lock on the reactor.
pub(crate) struct ReactorLock<'a> {
    reactor: &'a Reactor,
    events: MutexGuard<'a, Events>,
}

impl ReactorLock<'_> {
    /// Processes new events, blocking until the first event or the timeout.
    pub(crate) fn react(&mut self, timeout: Option<Duration>) -> Result<()> {
        #[cfg(feature = "tracing")]
        let span = tracing::trace_span!("react");
        #[cfg(feature = "tracing")]
        let _enter = span.enter();

        let mut wakers = Vec::new();

        // Process ready timers.
        let next_timer = self.reactor.process_timers(&mut wakers);

        // compute the timeout for blocking on I/O events.
        let timeout = match (next_timer, timeout) {
            (None, None) => None,
            (Some(t), None) | (None, Some(t)) => Some(t),
            (Some(a), Some(b)) => Some(a.min(b)),
        };

        // Bump the ticker before polling I/O.
        let tick = self
            .reactor
            .ticker
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);

        self.events.clear();

        let res = Ok(());
        #[cfg(false)]
        // Block on I/O events.
        let res = match self.reactor.poller.wait(&mut self.events, timeout) {
            // No I/O events occurred.
            Ok(0) => {
                if timeout != Some(Duration::from_secs(0)) {
                    // The non-zero timeout was hit so fire ready timers.
                    self.reactor.process_timers(&mut wakers);
                }
                Ok(())
            }

            // At least one I/O event occurred.
            Ok(_) => {
                // Iterate over sources in the event list.
                let sources = self.reactor.sources.lock().unwrap();

                for ev in self.events.iter() {
                    // Check if there is a source in the table with this key.
                    if let Some(source) = sources.get(ev.key) {
                        let mut state = source.state.lock().unwrap();

                        // Collect wakers if any event was emitted.
                        for &(dir, emitted) in &[(WRITE, ev.writable), (READ, ev.readable)] {
                            if emitted {
                                state[dir].tick = tick;
                                state[dir].drain_into(&mut wakers);
                            }
                        }

                        // Re-register if there are still writers or readers. This can happen if
                        // e.g. we were previously interested in both readability and writability,
                        // but only one of them was emitted.
                        if !state[READ].is_empty() || !state[WRITE].is_empty() {
                            // Create the event that we are interested in.
                            let event = {
                                let mut event = Event::none(source.key);
                                event.readable = !state[READ].is_empty();
                                event.writable = !state[WRITE].is_empty();
                                event
                            };

                            // Register interest in this event.
                            source.registration.modify(&self.reactor.poller, event)?;
                        }
                    }
                }

                Ok(())
            }

            // The syscall was interrupted.
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok(()),

            // An actual error occureed.
            Err(err) => Err(err),
        };

        
        // Wake up ready tasks.
        #[cfg(feature = "tracing")]
        tracing::trace!("{} ready wakers", wakers.len());
        for waker in wakers {
            waker.wake()
        }

        res
    }
}


/// A single timer operation.
enum TimerOp {
    Insert(Instant, usize, Waker),
    Remove(Instant, usize),
}