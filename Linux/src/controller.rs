//! What the window shows about the engine, kept apart from GTK so it can be tested. It mirrors
//! `macOS/Sources/Nasib/Engine.swift`.

use std::collections::VecDeque;

use crate::output_parser::{parse, parse_status, EngineEvent};
use crate::text::{
    CONNECTED_TITLE, NOTHING_WORKED_BODY, NOTHING_WORKED_TITLE, TUNNEL_DOWN_BODY, TUNNEL_DOWN_TITLE,
};

const RECENT_LINE_LIMIT: usize = 200;
const CLI_MISSING_PREFIX: &str = "windscribe-cli missing: ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Unknown,
    Down,
    Searching { location: String, protocol: String },
    Connected { location: String, protocol: String },
}

/// What a status read changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusUpdate {
    /// Something the window shows is different, so it needs redrawing.
    pub changed: bool,
    /// A connected tunnel just went down, so auto-reconnect can start the watch at once instead of
    /// waiting for the next minute's check.
    pub dropped: bool,
}

/// A desktop notification to post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

fn notification(title: &str, body: &str) -> Option<Notification> {
    Some(Notification {
        title: title.to_string(),
        body: body.to_string(),
    })
}

/// True when a tunnel the app showed as connected has gone down. A stopped search or a tunnel that was
/// never up is not a drop, so it doesn't start a reconnect the user didn't ask for.
pub fn tunnel_dropped(previous: &ConnectionState, current: &ConnectionState) -> bool {
    matches!(previous, ConnectionState::Connected { .. }) && *current == ConnectionState::Down
}

/// Whether the line at this position in the recent lines gets the alternating background. It counts from
/// the first line ever shown, so dropping the oldest line doesn't flip every row's stripe.
pub fn is_striped(index: usize, dropped_line_count: usize) -> bool {
    (index + dropped_line_count) % 2 == 1
}

pub struct Controller {
    pub state: ConnectionState,
    /// True while the engine runs, for any reason.
    pub is_busy: bool,
    /// True during a find, from Find Now or from an auto-reconnect check that found the tunnel down. A
    /// healthy check is busy for a moment but never finding, so the button doesn't flicker.
    pub is_finding: bool,
    pub is_surveying: bool,
    pub engine_missing: bool,
    pub cli_missing: bool,
    pub recent_lines: VecDeque<String>,
    /// How many lines have scrolled out of `recent_lines` since launch.
    pub dropped_line_count: usize,
}

impl Controller {
    pub fn new(engine_missing: bool) -> Controller {
        Controller {
            // Without the engine no status read ever lands, so it starts down instead of checking.
            state: if engine_missing {
                ConnectionState::Down
            } else {
                ConnectionState::Unknown
            },
            is_busy: false,
            is_finding: false,
            is_surveying: false,
            engine_missing,
            cli_missing: false,
            recent_lines: VecDeque::new(),
            dropped_line_count: 0,
        }
    }

    fn append(&mut self, line: &str) {
        self.recent_lines.push_back(line.to_string());
        while self.recent_lines.len() > RECENT_LINE_LIMIT {
            self.recent_lines.pop_front();
            self.dropped_line_count += 1;
        }
    }

    /// True when a new run may start: the engine is there and nothing else is running.
    pub fn can_run(&self) -> bool {
        !self.engine_missing && !self.is_busy
    }

    pub fn begin_find(&mut self) {
        self.is_busy = true;
        self.is_finding = true;
    }

    pub fn begin_survey(&mut self) {
        self.is_busy = true;
        self.is_surveying = true;
    }

    pub fn begin_watch(&mut self) {
        self.is_busy = true;
    }

    pub fn engine_exited(&mut self) {
        self.is_busy = false;
        self.is_finding = false;
        self.is_surveying = false;
    }

    /// Shows a line of a run's output and returns the notification it calls for, if any.
    pub fn handle_line(&mut self, line: &str) -> Option<Notification> {
        self.append(line);
        match parse(line) {
            EngineEvent::Trying { location, protocol } => {
                self.state = ConnectionState::Searching { location, protocol };
                None
            }
            EngineEvent::Connected { location, protocol } => {
                let body = format!("{location} / {protocol}");
                self.state = ConnectionState::Connected { location, protocol };
                notification(CONNECTED_TITLE, &body)
            }
            EngineEvent::NothingWorked => {
                self.state = ConnectionState::Down;
                notification(NOTHING_WORKED_TITLE, NOTHING_WORKED_BODY)
            }
            EngineEvent::TunnelDown => {
                self.state = ConnectionState::Down;
                self.is_finding = true;
                notification(TUNNEL_DOWN_TITLE, TUNNEL_DOWN_BODY)
            }
            EngineEvent::CliMissing(_) => {
                self.state = ConnectionState::Down;
                self.cli_missing = true;
                None
            }
            EngineEvent::Failed { .. }
            | EngineEvent::SearchStopped
            | EngineEvent::SurveyStopped
            | EngineEvent::SurveyFinished
            | EngineEvent::Phase(_)
            | EngineEvent::Other(_) => None,
        }
    }

    /// Every line shown since launch, including the ones that scrolled out.
    pub fn shown_line_total(&self) -> usize {
        self.dropped_line_count + self.recent_lines.len()
    }

    /// How many of the kept lines are new since the pane last showed `shown_through` lines in total.
    pub fn new_line_count(&self, shown_through: usize) -> usize {
        self.shown_line_total()
            .saturating_sub(shown_through)
            .min(self.recent_lines.len())
    }

    /// Shows what `--status` printed. Ignored while a run is going, since the run's own lines are newer.
    pub fn apply_status(&mut self, line: &str) -> StatusUpdate {
        if self.is_busy {
            return StatusUpdate::default();
        }
        let cli_missing = line.starts_with(CLI_MISSING_PREFIX);
        let cli_changed = cli_missing != self.cli_missing;
        self.cli_missing = cli_missing;
        let previous = std::mem::replace(
            &mut self.state,
            match parse_status(line) {
                Some(status) => ConnectionState::Connected {
                    location: status.location,
                    protocol: status.protocol,
                },
                None => ConnectionState::Down,
            },
        );
        StatusUpdate {
            changed: cli_changed || previous != self.state,
            dropped: tunnel_dropped(&previous, &self.state),
        }
    }
}
