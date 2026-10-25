//! Turns the engine's output lines, as `Engine/README.md` lists them, into events the window reacts to.

const TRYING_PREFIX: &str = "trying: ";
const CONNECTED_PREFIX: &str = "connected: ";
const FAILED_PREFIX: &str = "failed: ";
const NOTHING_WORKED_LINE: &str = "nothing worked";
const SURVEY_FINISHED_LINE: &str = "survey finished";
const SURVEY_STOPPED_LINE: &str = "survey stopped";
const SEARCH_STOPPED_LINE: &str = "search stopped";
const TUNNEL_DOWN_LINE: &str = "tunnel down";
const CLI_MISSING_PREFIX: &str = "windscribe-cli missing: ";
const PHASE_PREFIX: &str = "phase: ";
const PAIR_SEPARATOR: &str = " / ";

/// How an output line is colored. A failed attempt is routine while searching, so only "nothing worked"
/// gets the error tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTone {
    Primary,
    Success,
    Muted,
    Warning,
    Error,
    Accent,
    Detail,
    /// The header that starts each stage of a search.
    Phase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    Trying { location: String, protocol: String },
    Connected { location: String, protocol: String },
    Failed { location: String, protocol: String },
    NothingWorked,
    SurveyFinished,
    SurveyStopped,
    SearchStopped,
    TunnelDown,
    CliMissing(String),
    Phase(String),
    Other(String),
}

impl EngineEvent {
    pub fn tone(&self) -> OutputTone {
        match self {
            EngineEvent::Trying { .. } => OutputTone::Primary,
            EngineEvent::Connected { .. } => OutputTone::Success,
            EngineEvent::Failed { .. } => OutputTone::Muted,
            EngineEvent::TunnelDown => OutputTone::Warning,
            EngineEvent::NothingWorked | EngineEvent::CliMissing(_) => OutputTone::Error,
            EngineEvent::SearchStopped
            | EngineEvent::SurveyStopped
            | EngineEvent::SurveyFinished => OutputTone::Accent,
            EngineEvent::Phase(_) => OutputTone::Phase,
            EngineEvent::Other(_) => OutputTone::Detail,
        }
    }
}

/// What `nasib-engine --status` reports: the CLI's own location and protocol names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelStatus {
    pub location: String,
    pub protocol: String,
}

/// Splits "Location / protocol" at the last separator, since a location name may hold one too.
fn split_pair(text: &str) -> Option<(String, String)> {
    let (location, protocol) = text.rsplit_once(PAIR_SEPARATOR)?;
    Some((location.to_string(), protocol.to_string()))
}

fn pair_after(line: &str, prefix: &str) -> Option<(String, String)> {
    split_pair(line.strip_prefix(prefix)?)
}

pub fn parse(line: &str) -> EngineEvent {
    if let Some((location, protocol)) = pair_after(line, TRYING_PREFIX) {
        return EngineEvent::Trying { location, protocol };
    }
    if let Some((location, protocol)) = pair_after(line, CONNECTED_PREFIX) {
        return EngineEvent::Connected { location, protocol };
    }
    if let Some((location, protocol)) = pair_after(line, FAILED_PREFIX) {
        return EngineEvent::Failed { location, protocol };
    }
    match line {
        NOTHING_WORKED_LINE => return EngineEvent::NothingWorked,
        SURVEY_FINISHED_LINE => return EngineEvent::SurveyFinished,
        SURVEY_STOPPED_LINE => return EngineEvent::SurveyStopped,
        SEARCH_STOPPED_LINE => return EngineEvent::SearchStopped,
        TUNNEL_DOWN_LINE => return EngineEvent::TunnelDown,
        _ => {}
    }
    if let Some(path) = line.strip_prefix(CLI_MISSING_PREFIX) {
        return EngineEvent::CliMissing(path.to_string());
    }
    if let Some(stage) = line.strip_prefix(PHASE_PREFIX) {
        return EngineEvent::Phase(stage.to_string());
    }
    EngineEvent::Other(line.to_string())
}

/// Reads the one line `nasib-engine --status` prints. `None` for `down`, a missing CLI, or anything else
/// that isn't a fully connected tunnel.
pub fn parse_status(line: &str) -> Option<TunnelStatus> {
    let (location, protocol) = pair_after(line, CONNECTED_PREFIX)?;
    Some(TunnelStatus { location, protocol })
}
