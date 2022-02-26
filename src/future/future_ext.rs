use core::future::Future;

use crate::task::{Sleep, SleepUntil};

use super::{Delay, Timeout};
use crate::time::{Duration, Instant};

/// Extend `Future` with time-based operations.
pub trait FutureExt: Future {
    /// Await a future or times out after a duration of time.     
    fn timeout(self, dur: Duration) -> Timeout<Self, Sleep>
    where
        Self: Sized,
    {
        let deadline = crate::task::sleep(dur);
        Timeout::new(self, deadline)
    }

    /// Returns a future that delays execution for a specified time.
    fn delay<D: Future>(self, deadline: Instant) -> Delay<Self, SleepUntil>
    where
        Self: Sized,
    {
        let deadline = crate::task::sleep_until(deadline);
        Delay::new(self, deadline)
    }
}

impl<T> FutureExt for T where T: Future {}
