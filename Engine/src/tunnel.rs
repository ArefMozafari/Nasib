//! Connects, disconnects and checks the tunnel through the CLI, with the waits the Windscribe app needs.

use std::sync::Arc;
use std::time::Duration;

use crate::attempt_log::ServerPair;
use crate::blocked_site::BlockedSiteProbe;
use crate::stop::{StopSignal, Stopped};
use crate::timings::Timings;
use crate::tunnel_status::TunnelStatus;
use crate::windscribe_cli::{parse_ports, WindscribeCli};

pub struct Tunnel {
    cli: Arc<dyn WindscribeCli>,
    probe: Arc<dyn BlockedSiteProbe>,
    timings: Timings,
}

impl Tunnel {
    pub fn new(
        cli: Arc<dyn WindscribeCli>,
        probe: Arc<dyn BlockedSiteProbe>,
        timings: Timings,
    ) -> Tunnel {
        Tunnel {
            cli,
            probe,
            timings,
        }
    }

    pub fn read_status(&self) -> TunnelStatus {
        TunnelStatus::parse(&self.cli.read_status())
    }

    /// The protocol's ports as the CLI lists them, default first; empty when it lists none.
    pub fn ports(&self, protocol: &str) -> Vec<u16> {
        parse_ports(&self.cli.ports(protocol))
    }

    /// Disconnects and waits until the CLI reports Disconnected or an error. The app ignores a connect
    /// issued while it is still tearing down the previous tunnel, and it keeps reporting an error such as
    /// "Error: 11" after a failed connect, so an error ends the wait too. Returns false when time runs out.
    pub fn disconnect_and_wait(&self, stop: &StopSignal) -> Result<bool, Stopped> {
        self.cli.disconnect();
        let mut waited = Duration::ZERO;
        while waited < self.timings.disconnect_timeout {
            if self.read_status().is_down() {
                return Ok(true);
            }
            stop.wait(self.timings.status_poll_interval)?;
            waited += self.timings.status_poll_interval;
        }
        Ok(false)
    }

    /// Tears down any tunnel first, since the app drops a connect issued while one is up, then starts a
    /// non-blocking connect and polls, so a dead pair costs the connect timeout instead of the CLI's own
    /// 42 seconds. Ends with the settle delay, because the CLI can report Connected and drop seconds later.
    pub fn connect(&self, pair: &ServerPair, stop: &StopSignal) -> Result<(), Stopped> {
        if !self.read_status().is_down() {
            self.disconnect_and_wait(stop)?;
        }
        if stop.is_requested() {
            return Err(Stopped);
        }
        self.cli.connect(&pair.nickname, &pair.protocol, pair.port);
        let mut waited = Duration::ZERO;
        while waited < self.timings.connect_timeout {
            stop.wait(self.timings.status_poll_interval)?;
            waited += self.timings.status_poll_interval;
            if self.read_status().is_connected() {
                break;
            }
        }
        stop.wait(self.timings.settle_delay)
    }

    /// True only when the tunnel is up and the blocked site answers through it.
    pub fn works(&self, stop: &StopSignal) -> Result<bool, Stopped> {
        if !self.read_status().is_connected() {
            return Ok(false);
        }
        let probe = self.probe.clone();
        stop.run(move || probe.loads())
    }
}
