//! Something the engine did, with the exact line it prints for it. The apps read these lines, so their
//! text is a contract: see the output contract in `Engine/README.md`.

use std::path::PathBuf;

use crate::attempt_log::ServerPair;

const PAIR_SEPARATOR: &str = " / ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    /// An attempt on one datacenter and protocol has started.
    Trying(ServerPair),
    /// A blocked site loaded through the tunnel.
    Connected(ServerPair),
    /// An attempt ended without a blocked site loading. Normal while searching, not an error.
    Failed(ServerPair),
    /// A find or watch was stopped; the tunnel is torn down and nothing is reconnected.
    SearchStopped,
    /// A survey was stopped before it finished; the last pair that worked is reconnected.
    SurveyStopped,
    /// A survey tried every pair.
    SurveyFinished,
    /// A search tried every pair and none worked.
    NothingWorked,
    /// Watch found the tunnel down or not carrying traffic, and is about to search.
    TunnelDown,
    /// Another run holds the lock.
    SearchAlreadyRunning,
    /// The Windscribe CLI is not at its expected path.
    CliMissing(PathBuf),
    /// Any other progress line, such as the order the search will try things in.
    Message(String),
}

fn pair_text(prefix: &str, pair: &ServerPair) -> String {
    format!("{prefix}{}{PAIR_SEPARATOR}{}", pair.nickname, pair.protocol)
}

impl EngineEvent {
    pub fn text(&self) -> String {
        match self {
            EngineEvent::Trying(pair) => pair_text("trying: ", pair),
            EngineEvent::Connected(pair) => pair_text("connected: ", pair),
            EngineEvent::Failed(pair) => pair_text("failed: ", pair),
            EngineEvent::SearchStopped => "search stopped".to_string(),
            EngineEvent::SurveyStopped => "survey stopped".to_string(),
            EngineEvent::SurveyFinished => "survey finished".to_string(),
            EngineEvent::NothingWorked => "nothing worked".to_string(),
            EngineEvent::TunnelDown => "tunnel down".to_string(),
            EngineEvent::SearchAlreadyRunning => "search already running".to_string(),
            EngineEvent::CliMissing(path) => format!("windscribe-cli missing: {}", path.display()),
            EngineEvent::Message(message) => message.clone(),
        }
    }
}
