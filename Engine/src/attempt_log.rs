//! The attempt history in `log.csv` and the last pair that worked in `last_good`. Lines written before
//! ports were recorded have no port, and stay readable.

use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::paths::Paths;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
const FIELD_SEPARATOR: char = ',';
const PAIR_SEPARATOR: char = '|';
const PORT_SEPARATOR: char = ':';
const SUCCEEDED_VALUE: &str = "ok";
const FAILED_VALUE: &str = "fail";
/// Lines from before ports were recorded have four fields; newer lines add the port as a fifth.
const FIELD_COUNT_WITHOUT_PORT: usize = 4;
const FIELD_COUNT_WITH_PORT: usize = 5;
const LINE_END: u8 = b'\n';

/// Where attempt timestamps come from, so tests can fix them.
pub trait Clock: Send + Sync {
    /// The local time as `YYYY-MM-DDTHH:MM:SS`.
    fn local_timestamp(&self) -> String;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn local_timestamp(&self) -> String {
        chrono::Local::now().format(TIMESTAMP_FORMAT).to_string()
    }
}

/// One line of `log.csv`: when, which datacenter, protocol and port, and whether a blocked site loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRecord {
    pub timestamp: String,
    pub nickname: String,
    pub protocol: String,
    /// `None` for lines from before ports were recorded, and for attempts whose port wasn't known.
    pub port: Option<u16>,
    pub succeeded: bool,
}

/// A datacenter, protocol and port, such as the last pair that carried traffic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPair {
    pub nickname: String,
    pub protocol: String,
    /// `None` lets Windscribe pick the port.
    pub port: Option<u16>,
}

impl ServerPair {
    /// The protocol with its port, such as `tcp:443`, or the protocol alone when there is no port.
    pub fn label(&self) -> String {
        match self.port {
            Some(port) => format!("{}{PORT_SEPARATOR}{port}", self.protocol),
            None => self.protocol.clone(),
        }
    }
}

pub struct AttemptLog {
    paths: Paths,
    clock: Box<dyn Clock>,
}

/// Returns the result field's meaning, or `None` for anything else, such as a value cut off by a crash.
fn parse_result(field: &str) -> Option<bool> {
    match field {
        SUCCEEDED_VALUE => Some(true),
        FAILED_VALUE => Some(false),
        _ => None,
    }
}

fn parse_record(line: &str) -> Option<AttemptRecord> {
    let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
    let port = match fields.len() {
        FIELD_COUNT_WITHOUT_PORT => None,
        FIELD_COUNT_WITH_PORT if fields[4].is_empty() => None,
        FIELD_COUNT_WITH_PORT => Some(fields[4].parse::<u16>().ok()?),
        _ => return None,
    };
    Some(AttemptRecord {
        timestamp: fields[0].to_string(),
        nickname: fields[1].to_string(),
        protocol: fields[2].to_string(),
        port,
        succeeded: parse_result(fields[3])?,
    })
}

/// Reads `<protocol>` or `<protocol>:<port>`; `None` when the port isn't a number.
fn parse_label(label: &str) -> Option<(String, Option<u16>)> {
    match label.split_once(PORT_SEPARATOR) {
        Some((protocol, port)) => Some((protocol.to_string(), Some(port.parse().ok()?))),
        None => Some((label.to_string(), None)),
    }
}

impl AttemptLog {
    pub fn new(paths: Paths, clock: Box<dyn Clock>) -> AttemptLog {
        AttemptLog { paths, clock }
    }

    /// Appends one attempt. A log cut off mid-line, by a crash, gets its line ended first, so the new
    /// attempt stays readable. A write that fails is dropped: the search matters more than its history.
    pub fn record(&self, pair: &ServerPair, succeeded: bool) {
        let path = self.paths.log_file();
        let needs_line_end =
            fs::read(&path).is_ok_and(|bytes| bytes.last().is_some_and(|last| *last != LINE_END));
        let result = if succeeded {
            SUCCEEDED_VALUE
        } else {
            FAILED_VALUE
        };
        let port = pair.port.map(|port| port.to_string()).unwrap_or_default();
        let mut line = format!(
            "{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{result}{FIELD_SEPARATOR}{port}\n",
            self.clock.local_timestamp(),
            pair.nickname,
            pair.protocol
        );
        if needs_line_end {
            line.insert(0, char::from(LINE_END));
        }
        let _ = fs::create_dir_all(&self.paths.state_directory);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(line.as_bytes());
        }
    }

    /// Returns the last lines of the log, oldest first, skipping any that are malformed.
    pub fn read_recent(&self, count: usize) -> Vec<AttemptRecord> {
        let Ok(bytes) = fs::read(self.paths.log_file()) else {
            return Vec::new();
        };
        let text = String::from_utf8_lossy(&bytes);
        let lines: Vec<&str> = text.lines().collect();
        lines[lines.len().saturating_sub(count)..]
            .iter()
            .filter_map(|line| parse_record(line))
            .collect()
    }

    pub fn read_last_good(&self) -> Option<ServerPair> {
        let text = fs::read_to_string(self.paths.last_good_file()).ok()?;
        let parts: Vec<&str> = text.trim().split(PAIR_SEPARATOR).collect();
        match parts.as_slice() {
            [nickname, label] if !nickname.is_empty() && !label.is_empty() => {
                let (protocol, port) = parse_label(label)?;
                Some(ServerPair {
                    nickname: nickname.to_string(),
                    protocol,
                    port,
                })
            }
            _ => None,
        }
    }

    pub fn write_last_good(&self, pair: &ServerPair) {
        let _ = fs::create_dir_all(&self.paths.state_directory);
        let _ = fs::write(
            self.paths.last_good_file(),
            format!("{}{PAIR_SEPARATOR}{}\n", pair.nickname, pair.label()),
        );
    }
}
