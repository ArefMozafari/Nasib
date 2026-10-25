//! Finds a Windscribe datacenter and protocol that carries traffic right now, connects to it, and keeps
//! it connected, in three modes: find searches once, watch searches only when the tunnel stopped working,
//! and survey tries every pair.

use std::sync::Arc;

use crate::attempt_log::{AttemptLog, ServerPair};
use crate::catalog;
use crate::client_log;
use crate::configuration::Configuration;
use crate::datacenter::Datacenter;
use crate::events::EngineEvent;
use crate::paths::Paths;
use crate::ranking::{self, PortOrder, RANK_WINDOW_LINES};
use crate::reachability::{self, ReachabilityProbe};
use crate::run_lock::{LockRefusal, RunLock};
use crate::stop::{StopSignal, Stopped};
use crate::tunnel::Tunnel;

const LIST_SEPARATOR: &str = ", ";

/// How a run ended. The binary maps it to an exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A pair works, or a watch found the tunnel healthy.
    Succeeded,
    /// Nothing worked, or there was nothing to try.
    Failed,
    Stopped,
    AlreadyRunning,
}

pub type EventSink = Box<dyn Fn(EngineEvent) + Send + Sync>;

pub struct Finder {
    paths: Paths,
    /// The bundled datacenters; tests pass their own.
    bundled: Vec<Datacenter>,
    tunnel: Tunnel,
    reachability_probe: Arc<dyn ReachabilityProbe>,
    attempt_log: AttemptLog,
    events: EventSink,
}

/// The settings and datacenters one run works from, read once when it starts.
struct SearchScope {
    configuration: Configuration,
    datacenters: Vec<Datacenter>,
}

impl Finder {
    pub fn new(
        paths: Paths,
        bundled: Vec<Datacenter>,
        tunnel: Tunnel,
        reachability_probe: Arc<dyn ReachabilityProbe>,
        attempt_log: AttemptLog,
        events: EventSink,
    ) -> Finder {
        Finder {
            paths,
            bundled,
            tunnel,
            reachability_probe,
            attempt_log,
            events,
        }
    }

    fn raise(&self, event: EngineEvent) {
        (self.events)(event);
    }

    /// One attempt: connect, check that a blocked site loads, and log the result. A stop abandons the
    /// attempt without logging it. An attempt without a port logs the port Windscribe chose, as the
    /// status reports it.
    fn try_pair(&self, pair: &ServerPair, stop: &StopSignal) -> Result<bool, Stopped> {
        self.raise(EngineEvent::Trying(pair.clone()));
        self.tunnel.connect(pair, stop)?;
        let works = self.tunnel.works(stop)?;
        let pair = match pair.port {
            Some(_) => pair.clone(),
            None => ServerPair {
                port: self.tunnel.read_status().port,
                ..pair.clone()
            },
        };
        if works {
            self.attempt_log.record(&pair, true);
            self.attempt_log.write_last_good(&pair);
            self.raise(EngineEvent::Connected(pair));
            return Ok(true);
        }
        self.attempt_log.record(&pair, false);
        self.raise(EngineEvent::Failed(pair));
        self.tunnel.disconnect_and_wait(stop)?;
        Ok(false)
    }

    fn reconnect_last_good(&self, stop: &StopSignal) -> Result<bool, Stopped> {
        match self.attempt_log.read_last_good() {
            Some(pair) => self.try_pair(&pair, stop),
            None => Ok(false),
        }
    }

    /// One phase: the protocol on one port, on every datacenter in order. Stops at the first pair that
    /// works unless surveying.
    fn try_port(
        &self,
        protocol: &str,
        port: Option<u16>,
        nicknames: &[String],
        is_survey: bool,
        stop: &StopSignal,
    ) -> Result<bool, Stopped> {
        let phase = match port {
            Some(port) => format!("{protocol} on port {port}"),
            None => protocol.to_string(),
        };
        self.raise(EngineEvent::Phase(phase));
        for nickname in nicknames {
            let pair = ServerPair {
                nickname: nickname.clone(),
                protocol: protocol.to_string(),
                port,
            };
            if self.try_pair(&pair, stop)? {
                if !is_survey {
                    return Ok(true);
                }
                self.tunnel.disconnect_and_wait(stop)?;
            }
        }
        Ok(false)
    }

    /// Tries pairs best-first: every reachable datacenter by rank, on each protocol in the configured
    /// order. Stops at the first pair that works unless surveying, which logs every result and goes on.
    fn search(
        &self,
        scope: &SearchScope,
        is_survey: bool,
        stop: &StopSignal,
    ) -> Result<bool, Stopped> {
        if !is_survey && self.attempt_log.read_last_good().is_some() {
            self.raise(EngineEvent::Phase("last pair that worked".to_string()));
            if self.reconnect_last_good(stop)? {
                return Ok(true);
            }
        }
        self.raise(EngineEvent::Phase("reachability".to_string()));
        let mut candidates =
            reachability::reachable(&scope.datacenters, self.reachability_probe.clone(), stop)?;
        let nothing_answered = candidates.is_empty();
        if nothing_answered {
            // The filter may be wrong on a bad day, so fall back to trying everything.
            self.raise(EngineEvent::Message(
                "no address answered; trying all locations".to_string(),
            ));
            candidates = scope.datacenters.clone();
        }
        let names: Vec<String> = candidates
            .into_iter()
            .map(|datacenter| datacenter.nickname)
            .collect();
        let recent = self.attempt_log.read_recent(RANK_WINDOW_LINES);
        let (nicknames, order_source) = if scope.configuration.rank_by_success {
            (
                ranking::rank(&names, &recent),
                "from Settings, by recent success",
            )
        } else {
            (names, "from Settings")
        };
        let protocols = &scope.configuration.protocol_order;
        self.raise(EngineEvent::Message(format!(
            "protocol order (from Settings): {}",
            protocols.join(LIST_SEPARATOR)
        )));
        self.raise(EngineEvent::Message(format!(
            "location order ({order_source}): {}",
            nicknames.join(LIST_SEPARATOR)
        )));
        // Read once per run, before any attempt: the CLI refuses a second command while one is running.
        let port_orders: Vec<(&String, PortOrder)> = protocols
            .iter()
            .map(|protocol| {
                let ports = self.tunnel.ports(protocol);
                (protocol, ranking::rank_ports(protocol, &ports, &recent))
            })
            .collect();
        for (protocol, order) in &port_orders {
            if self.try_port(protocol, order.preferred, &nicknames, is_survey, stop)? {
                return Ok(true);
            }
        }
        // Reached only when nothing worked on the preferred ports, or when surveying every port. When no
        // address answered a knock, the network itself is likely down, and every other port would fail
        // too, for hours; the connectivity check tracked in #12 will handle that case properly.
        if nothing_answered && !is_survey {
            self.raise(EngineEvent::Message(
                "no address answered, so the network may be down; skipping the other ports"
                    .to_string(),
            ));
        }
        let second_pass = if nothing_answered && !is_survey {
            &[][..]
        } else {
            &port_orders[..]
        };
        for (protocol, order) in second_pass {
            for port in &order.others {
                if self.try_port(protocol, Some(*port), &nicknames, is_survey, stop)? {
                    return Ok(true);
                }
            }
        }
        if is_survey {
            self.raise(EngineEvent::SurveyFinished);
            return self.reconnect_last_good(stop);
        }
        self.raise(EngineEvent::NothingWorked);
        Ok(false)
    }

    /// Takes the run lock and reads the settings and datacenters, then runs the given search.
    fn run_exclusive(&self, run: impl FnOnce(&SearchScope) -> Outcome) -> Outcome {
        let lock_file = self.paths.lock_file();
        let _lock = match RunLock::acquire(&lock_file) {
            Ok(lock) => lock,
            Err(LockRefusal::HeldByAnotherRun) => {
                self.raise(EngineEvent::SearchAlreadyRunning);
                return Outcome::AlreadyRunning;
            }
            Err(LockRefusal::BlockedByFolder) => {
                self.raise(EngineEvent::Message(format!(
                    "{} is a folder with files in it; remove it so a search can start",
                    lock_file.display()
                )));
                return Outcome::Failed;
            }
            Err(LockRefusal::Unavailable) => {
                self.raise(EngineEvent::Message(format!(
                    "could not create {}",
                    lock_file.display()
                )));
                return Outcome::Failed;
            }
        };
        let configuration = Configuration::load(&self.paths.configuration_file());
        let logged = client_log::read_directory(&self.paths.windscribe_log_directory);
        let datacenters = configuration.select(&catalog::merge(self.bundled.clone(), logged));
        if datacenters.is_empty() {
            self.raise(EngineEvent::Message(
                "no datacenters match the locations in Settings".to_string(),
            ));
            return Outcome::Failed;
        }
        run(&SearchScope {
            configuration,
            datacenters,
        })
    }

    /// Ends a stopped find or watch: the tunnel is torn down and the last good pair is deliberately not
    /// reconnected, because that would usually retry the server whose failure started the search.
    fn finish_stopped_search(&self, stop: &StopSignal) -> Outcome {
        self.raise(EngineEvent::SearchStopped);
        let _ = self.tunnel.disconnect_and_wait(&stop.shield());
        Outcome::Stopped
    }

    fn outcome_of(found: bool) -> Outcome {
        if found {
            Outcome::Succeeded
        } else {
            Outcome::Failed
        }
    }

    /// Tries the last pair that worked, then searches, and stops at the first pair that carries traffic.
    pub fn find(&self, stop: &StopSignal) -> Outcome {
        self.run_exclusive(|scope| match self.search(scope, false, stop) {
            Ok(found) => Finder::outcome_of(found),
            Err(Stopped) => self.finish_stopped_search(stop),
        })
    }

    /// Does nothing when the tunnel is up and a blocked site loads through it; otherwise searches as
    /// find does, and stops as find does. A stop during the health check itself, such as the app quitting
    /// in that moment, leaves the tunnel as it is: nothing has been found wrong with it yet.
    pub fn watch(&self, stop: &StopSignal) -> Outcome {
        self.run_exclusive(|scope| {
            match self.tunnel.works(stop) {
                Ok(true) => return Outcome::Succeeded,
                Ok(false) => {}
                Err(Stopped) => return Outcome::Stopped,
            }
            self.raise(EngineEvent::TunnelDown);
            match self.search(scope, false, stop) {
                Ok(found) => Finder::outcome_of(found),
                Err(Stopped) => self.finish_stopped_search(stop),
            }
        })
    }

    /// Tries every pair and logs each result without stopping at a success, then reconnects the last pair
    /// that worked. A stop reconnects that pair too, so the user is put back online; that clean-up can't
    /// itself be stopped.
    pub fn survey(&self, stop: &StopSignal) -> Outcome {
        self.run_exclusive(|scope| match self.search(scope, true, stop) {
            Ok(reconnected) => Finder::outcome_of(reconnected),
            Err(Stopped) => {
                self.raise(EngineEvent::SurveyStopped);
                let shielded = stop.shield();
                let _ = self.tunnel.disconnect_and_wait(&shielded);
                let _ = self.reconnect_last_good(&shielded);
                Outcome::Stopped
            }
        })
    }
}
