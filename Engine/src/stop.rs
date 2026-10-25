//! The stop request every wait in the engine listens for, so Stop takes effect at once instead of
//! after whatever timeout is running.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

/// How often running work checks for a stop; short enough that Stop feels immediate.
const RUN_CHECK_INTERVAL: Duration = Duration::from_millis(20);

/// Returned by a wait or a probe that gave up because a stop was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopped;

#[derive(Debug, Default)]
struct StopState {
    requested: Mutex<bool>,
    changed: Condvar,
}

/// A shared stop flag. Clones share the flag; a shielded signal has its own and never sets it.
#[derive(Debug, Clone, Default)]
pub struct StopSignal {
    state: Arc<StopState>,
    is_shielded: bool,
}

impl StopSignal {
    pub fn new() -> StopSignal {
        StopSignal::default()
    }

    /// Asks every wait on this signal to give up. Does nothing on a shielded signal.
    pub fn request(&self) {
        if self.is_shielded {
            return;
        }
        let mut requested = self
            .state
            .requested
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *requested = true;
        self.state.changed.notify_all();
    }

    pub fn is_requested(&self) -> bool {
        *self
            .state
            .requested
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Sleeps for the duration, or returns `Err(Stopped)` as soon as a stop is requested.
    pub fn wait(&self, duration: Duration) -> Result<(), Stopped> {
        let deadline = Instant::now() + duration;
        let mut requested = self
            .state
            .requested
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        loop {
            if *requested {
                return Err(Stopped);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(());
            }
            requested = self
                .state
                .changed
                .wait_timeout(requested, deadline - now)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }

    /// Runs blocking work, such as a probe, on its own thread and returns its result, or `Err(Stopped)`
    /// as soon as a stop is requested. Stopped work is abandoned; its own timeout ends it later.
    pub fn run<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, Stopped> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let _ = sender.send(work());
        });
        loop {
            match receiver.recv_timeout(RUN_CHECK_INTERVAL) {
                Ok(result) => return Ok(result),
                Err(RecvTimeoutError::Timeout) if self.is_requested() => return Err(Stopped),
                Err(RecvTimeoutError::Timeout) => continue,
                // The work panicked; treat it as stopped rather than taking the engine down with it.
                Err(RecvTimeoutError::Disconnected) => return Err(Stopped),
            }
        }
    }

    /// Returns a signal that ignores every request, for the clean-up after a stop, which must always finish.
    pub fn shield(&self) -> StopSignal {
        StopSignal {
            state: Arc::default(),
            is_shielded: true,
        }
    }
}
