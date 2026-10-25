//! Test helpers shared by the engine's test files. Each test file includes this module, so any one file
//! uses only part of it.
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, process, thread};

use nasib_engine::blocked_site::BlockedSiteProbe;
use nasib_engine::reachability::ReachabilityProbe;
use nasib_engine::windscribe_cli::WindscribeCli;

static NEXT_DIRECTORY_INDEX: AtomicUsize = AtomicUsize::new(0);

/// How long a hanging fake probe blocks; far longer than any test waits.
const HANG_DURATION: Duration = Duration::from_secs(30);
pub const STATUS_CALL: &str = "status";
pub const DISCONNECT_CALL: &str = "disconnect";
pub const ERROR_CODE: &str = "11";

/// A throwaway folder under the system temp folder, removed when dropped.
pub struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    pub fn new() -> TemporaryDirectory {
        let index = NEXT_DIRECTORY_INDEX.fetch_add(1, Ordering::SeqCst);
        let path = env::temp_dir().join(format!("nasib-engine-test-{}-{index}", process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TemporaryDirectory(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakeTunnelState {
    Disconnected,
    Connected,
    Error,
}

type PairRule = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;
type PairHook = Box<dyn Fn(&str, &str) + Send + Sync>;

pub struct FakeCliState {
    pub state: FakeTunnelState,
    pub connected_location: String,
    /// Decides whether a connect to the pair brings the tunnel up.
    pub pair_connects: PairRule,
    /// When true, a failed connect leaves the error state, and a disconnect does not clear it.
    pub error_is_sticky: bool,
    /// When true, the CLI reports Connected whatever it is told.
    pub stuck_connected: bool,
    /// Called before each connect, so a test can stop a search part way through.
    pub before_connect: Option<PairHook>,
    /// Every call in order: "status", "connect <nickname> <protocol>", "disconnect".
    pub calls: Vec<String>,
}

/// Mimics the real Windscribe app: a connect issued while a tunnel is up tears the old tunnel down and
/// drops the new request, leaving nothing connected, and a failed connect can leave a sticky error behind.
pub struct FakeCli {
    pub inner: Mutex<FakeCliState>,
}

impl FakeCli {
    pub fn new() -> Arc<FakeCli> {
        Arc::new(FakeCli {
            inner: Mutex::new(FakeCliState {
                state: FakeTunnelState::Disconnected,
                connected_location: "Dubai - Khalifa".to_string(),
                pair_connects: Box::new(|_, _| true),
                error_is_sticky: false,
                stuck_connected: false,
                before_connect: None,
                calls: Vec::new(),
            }),
        })
    }

    pub fn with(&self, change: impl FnOnce(&mut FakeCliState)) {
        change(&mut self.inner.lock().unwrap());
    }

    pub fn state(&self) -> FakeTunnelState {
        self.inner.lock().unwrap().state
    }

    pub fn connected_location(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        (inner.state == FakeTunnelState::Connected || inner.stuck_connected)
            .then(|| inner.connected_location.clone())
    }

    pub fn calls(&self) -> Vec<String> {
        self.inner.lock().unwrap().calls.clone()
    }

    pub fn connect_calls(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|call| call.starts_with("connect "))
            .collect()
    }

    pub fn count_calls(&self, name: &str) -> usize {
        self.calls().iter().filter(|call| *call == name).count()
    }

    /// The number of status reads after the last call with the given name.
    pub fn status_reads_after_last(&self, name_prefix: &str) -> usize {
        let calls = self.calls();
        let last = calls
            .iter()
            .rposition(|call| call.starts_with(name_prefix))
            .expect("the call happened");
        calls[last + 1..]
            .iter()
            .filter(|call| *call == STATUS_CALL)
            .count()
    }
}

impl WindscribeCli for FakeCli {
    fn read_status(&self) -> String {
        let mut inner = self.inner.lock().unwrap();
        inner.calls.push(STATUS_CALL.to_string());
        if inner.stuck_connected {
            return format!(
                "Connect state: Connected: {}\nProtocol: IKEv2:500\n",
                inner.connected_location
            );
        }
        match inner.state {
            FakeTunnelState::Connected => {
                format!(
                    "Connect state: Connected: {}\nProtocol: IKEv2:500\n",
                    inner.connected_location
                )
            }
            FakeTunnelState::Error => format!("Connect state: Error: {ERROR_CODE}\n"),
            FakeTunnelState::Disconnected => "Connect state: Disconnected\n".to_string(),
        }
    }

    fn connect(&self, nickname: &str, protocol: &str) {
        let hook = self.inner.lock().unwrap().before_connect.take();
        if let Some(hook) = &hook {
            hook(nickname, protocol);
        }
        let mut inner = self.inner.lock().unwrap();
        inner.before_connect = hook;
        inner.calls.push(format!("connect {nickname} {protocol}"));
        if inner.state == FakeTunnelState::Connected {
            inner.state = FakeTunnelState::Disconnected;
        } else if (inner.pair_connects)(nickname, protocol) {
            inner.state = FakeTunnelState::Connected;
            inner.connected_location = nickname.to_string();
        } else {
            inner.state = if inner.error_is_sticky {
                FakeTunnelState::Error
            } else {
                FakeTunnelState::Disconnected
            };
        }
    }

    fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.calls.push(DISCONNECT_CALL.to_string());
        if !(inner.state == FakeTunnelState::Error && inner.error_is_sticky) {
            inner.state = FakeTunnelState::Disconnected;
        }
    }
}

type LocationRule = Box<dyn Fn(&str) -> bool + Send + Sync>;
type Hook = Box<dyn Fn() + Send + Sync>;

/// Loads through whatever location the fake CLI is connected to, as the rule decides.
pub struct FakeBlockedSiteProbe {
    cli: Arc<FakeCli>,
    pub loads_through: Mutex<LocationRule>,
    /// When set, the probe calls it and then hangs, like a request on a dead network.
    pub hangs_after: Mutex<Option<Hook>>,
    pub probe_calls: AtomicUsize,
}

impl FakeBlockedSiteProbe {
    pub fn new(cli: Arc<FakeCli>) -> Arc<FakeBlockedSiteProbe> {
        Arc::new(FakeBlockedSiteProbe {
            cli,
            loads_through: Mutex::new(Box::new(|_| true)),
            hangs_after: Mutex::new(None),
            probe_calls: AtomicUsize::new(0),
        })
    }

    pub fn set_loads_through(&self, rule: impl Fn(&str) -> bool + Send + Sync + 'static) {
        *self.loads_through.lock().unwrap() = Box::new(rule);
    }

    pub fn set_hangs_after(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.hangs_after.lock().unwrap() = Some(Box::new(hook));
    }
}

impl BlockedSiteProbe for FakeBlockedSiteProbe {
    fn loads(&self) -> bool {
        self.probe_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(hook) = self.hangs_after.lock().unwrap().as_ref() {
            hook();
            thread::sleep(HANG_DURATION);
        }
        match self.cli.connected_location() {
            Some(location) => (self.loads_through.lock().unwrap())(&location),
            None => false,
        }
    }
}

/// Answers for the given addresses, or for every address when built with `answering_all`.
pub struct FakeReachabilityProbe {
    answering: Option<HashSet<String>>,
}

impl FakeReachabilityProbe {
    pub fn answering_all() -> Arc<FakeReachabilityProbe> {
        Arc::new(FakeReachabilityProbe { answering: None })
    }

    pub fn answering(addresses: &[&str]) -> Arc<FakeReachabilityProbe> {
        Arc::new(FakeReachabilityProbe {
            answering: Some(
                addresses
                    .iter()
                    .map(|address| address.to_string())
                    .collect(),
            ),
        })
    }
}

impl ReachabilityProbe for FakeReachabilityProbe {
    fn answers(&self, address: &str) -> bool {
        self.answering
            .as_ref()
            .is_none_or(|answering| answering.contains(address))
    }
}
