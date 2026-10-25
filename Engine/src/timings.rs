//! How long the engine waits at each step. The defaults are the ones the apps have always used.

use std::env;
use std::time::Duration;

/// Divides every timing, so end-to-end tests can run a whole search in milliseconds. The apps never set it.
const TIME_SCALE_VARIABLE: &str = "NASIB_TIME_SCALE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timings {
    /// How long the blocked-site request may take before the tunnel counts as not working.
    pub probe_timeout: Duration,
    /// The CLI itself waits about 42 seconds per failed attempt; a working pair answers within this.
    pub connect_timeout: Duration,
    /// The CLI can report Connected and drop seconds later, so the engine waits this long before trusting it.
    pub settle_delay: Duration,
    /// The longest the engine waits for a tunnel to finish tearing down.
    pub disconnect_timeout: Duration,
    /// How often the engine reads the CLI status while it waits for a state change.
    pub status_poll_interval: Duration,
    /// How long one TCP knock on a datacenter address may take.
    pub reach_timeout: Duration,
}

impl Default for Timings {
    fn default() -> Self {
        Timings {
            probe_timeout: Duration::from_secs(8),
            connect_timeout: Duration::from_secs(10),
            settle_delay: Duration::from_secs(2),
            disconnect_timeout: Duration::from_secs(15),
            status_poll_interval: Duration::from_secs(1),
            reach_timeout: Duration::from_secs(2),
        }
    }
}

impl Timings {
    /// Returns these timings divided by the given divisor.
    pub fn scaled(self, divisor: f64) -> Timings {
        let scale = |duration: Duration| duration.div_f64(divisor);
        Timings {
            probe_timeout: scale(self.probe_timeout),
            connect_timeout: scale(self.connect_timeout),
            settle_delay: scale(self.settle_delay),
            disconnect_timeout: scale(self.disconnect_timeout),
            status_poll_interval: scale(self.status_poll_interval),
            reach_timeout: scale(self.reach_timeout),
        }
    }

    /// Returns the defaults, scaled down when the test variable holds a positive number.
    pub fn from_environment() -> Timings {
        let divisor = env::var(TIME_SCALE_VARIABLE)
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| *value > 0.0);
        match divisor {
            Some(divisor) => Timings::default().scaled(divisor),
            None => Timings::default(),
        }
    }
}
