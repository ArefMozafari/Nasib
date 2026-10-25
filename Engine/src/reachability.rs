//! Drops datacenters where no address answers at all. A blackholed address cannot work on any protocol,
//! so skipping it loses nothing, and knocking on every address at once takes about two seconds.

use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::datacenter::Datacenter;
use crate::stop::{StopSignal, Stopped};

/// Every Windscribe datacenter listens on HTTPS, whichever protocol the tunnel uses.
pub const REACH_PORT: u16 = 443;

pub trait ReachabilityProbe: Send + Sync {
    /// True when the address accepts a TCP connection within the probe's timeout.
    fn answers(&self, address: &str) -> bool;
}

pub struct TcpReachabilityProbe {
    port: u16,
    timeout: Duration,
}

impl TcpReachabilityProbe {
    pub fn new(port: u16, timeout: Duration) -> TcpReachabilityProbe {
        TcpReachabilityProbe { port, timeout }
    }
}

impl ReachabilityProbe for TcpReachabilityProbe {
    fn answers(&self, address: &str) -> bool {
        let Ok(address) = address.parse::<IpAddr>() else {
            return false;
        };
        TcpStream::connect_timeout(&SocketAddr::new(address, self.port), self.timeout).is_ok()
    }
}

/// Returns the datacenters with at least one answering address, in the order given.
pub fn reachable(
    datacenters: &[Datacenter],
    probe: Arc<dyn ReachabilityProbe>,
    stop: &StopSignal,
) -> Result<Vec<Datacenter>, Stopped> {
    let owned = datacenters.to_vec();
    stop.run(move || {
        let answered: Vec<bool> = thread::scope(|scope| {
            let knocks: Vec<Vec<_>> = owned
                .iter()
                .map(|datacenter| {
                    datacenter
                        .addresses
                        .iter()
                        .map(|address| {
                            let probe = &probe;
                            scope.spawn(move || probe.answers(address))
                        })
                        .collect()
                })
                .collect();
            knocks
                .into_iter()
                .map(|address_knocks| {
                    // Every knock is joined, so none outlives the scope even after one answers.
                    let results: Vec<bool> = address_knocks
                        .into_iter()
                        .map(|knock| knock.join().unwrap_or(false))
                        .collect();
                    results.into_iter().any(|answered| answered)
                })
                .collect()
        });
        owned
            .into_iter()
            .zip(answered)
            .filter_map(|(datacenter, answered)| answered.then_some(datacenter))
            .collect()
    })
}
