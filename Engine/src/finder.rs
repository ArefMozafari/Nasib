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
use crate::ranking::{self, RANK_WINDOW_LINES};
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
    seed: Vec<Datacenter>,
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
        seed: Vec<Datacenter>,
        tunnel: Tunnel,
        reachability_probe: Arc<dyn ReachabilityProbe>,
        attempt_log: AttemptLog,
        events: EventSink,
    ) -> Finder {
        Finder {
            paths,
            seed,
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
    /// attempt without logging it.
    fn try_pair(&self, pair: &ServerPair, stop: &StopSignal) -> Result<bool, Stopped> {
        self.raise(EngineEvent::Trying(pair.clone()));
        self.tunnel.connect(&pair.nickname, &pair.protocol, stop)?;
        if self.tunnel.works(stop)? {
            self.attempt_log
                .record(&pair.nickname, &pair.protocol, true);
            self.attempt_log.write_last_good(pair);
            self.raise(EngineEvent::Connected(pair.clone()));
            return Ok(true);
        }
        self.attempt_log
            .record(&pair.nickname, &pair.protocol, false);
        self.raise(EngineEvent::Failed(pair.clone()));
        self.tunnel.disconnect_and_wait(stop)?;
        Ok(false)
    }

    fn reconnect_last_good(&self, stop: &StopSignal) -> Result<bool, Stopped> {
        match self.attempt_log.read_last_good() {
            Some(pair) => self.try_pair(&pair, stop),
            None => Ok(false),
        }
    }

    /// Tries pairs best-first: every reachable datacenter by rank, on each protocol in the configured
    /// order. Stops at the first pair that works unless surveying, which logs every result and goes on.
    fn search(
        &self,
        scope: &SearchScope,
        is_survey: bool,
        stop: &StopSignal,
    ) -> Result<bool, Stopped> {
        if !is_survey && self.reconnect_last_good(stop)? {
            return Ok(true);
        }
        let mut candidates =
            reachability::reachable(&scope.datacenters, self.reachability_probe.clone(), stop)?;
        if candidates.is_empty() {
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
        let nicknames = ranking::rank(&names, &self.attempt_log.read_recent(RANK_WINDOW_LINES));
        let protocols = &scope.configuration.protocol_order;
        self.raise(EngineEvent::Message(format!(
            "protocol order (from Settings): {}",
            protocols.join(LIST_SEPARATOR)
        )));
        self.raise(EngineEvent::Message(format!(
            "location order (by recent success): {}",
            nicknames.join(LIST_SEPARATOR)
        )));
        for protocol in protocols {
            for nickname in &nicknames {
                let pair = ServerPair {
                    nickname: nickname.clone(),
                    protocol: protocol.clone(),
                };
                if self.try_pair(&pair, stop)? {
                    if !is_survey {
                        return Ok(true);
                    }
                    self.tunnel.disconnect_and_wait(stop)?;
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
        let datacenters = catalog::merge(self.seed.clone(), logged, &configuration);
        if datacenters.is_empty() {
            self.raise(EngineEvent::Message(
                "no datacenters match the cities in Settings".to_string(),
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
