//! The attempt history in `log.csv` and the last pair that worked in `last_good`, in the formats the
//! apps have always used.

use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::paths::Paths;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
const FIELD_SEPARATOR: char = ',';
const PAIR_SEPARATOR: char = '|';
const SUCCEEDED_VALUE: &str = "ok";
const FAILED_VALUE: &str = "fail";
const FIELD_COUNT: usize = 4;
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

/// One line of `log.csv`: when, which datacenter, which protocol, and whether a blocked site loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRecord {
    pub timestamp: String,
    pub nickname: String,
    pub protocol: String,
    pub succeeded: bool,
}

/// A datacenter and protocol, such as the last pair that carried traffic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPair {
    pub nickname: String,
    pub protocol: String,
}

pub struct AttemptLog {
    paths: Paths,
    clock: Box<dyn Clock>,
}

fn parse_record(line: &str) -> Option<AttemptRecord> {
    let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
    if fields.len() != FIELD_COUNT {
        return None;
    }
    Some(AttemptRecord {
        timestamp: fields[0].to_string(),
        nickname: fields[1].to_string(),
        protocol: fields[2].to_string(),
        succeeded: fields[3] == SUCCEEDED_VALUE,
    })
}

impl AttemptLog {
    pub fn new(paths: Paths, clock: Box<dyn Clock>) -> AttemptLog {
        AttemptLog { paths, clock }
    }

    /// Appends one attempt. A log cut off mid-line, by a crash, gets its line ended first, so the new
    /// attempt stays readable. A write that fails is dropped: the search matters more than its history.
    pub fn record(&self, nickname: &str, protocol: &str, succeeded: bool) {
        let path = self.paths.log_file();
        let needs_line_end =
            fs::read(&path).is_ok_and(|bytes| bytes.last().is_some_and(|last| *last != LINE_END));
        let result = if succeeded {
            SUCCEEDED_VALUE
        } else {
            FAILED_VALUE
        };
        let mut line = format!(
            "{}{FIELD_SEPARATOR}{nickname}{FIELD_SEPARATOR}{protocol}{FIELD_SEPARATOR}{result}\n",
            self.clock.local_timestamp()
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
            [nickname, protocol] if !nickname.is_empty() && !protocol.is_empty() => {
                Some(ServerPair {
                    nickname: nickname.to_string(),
                    protocol: protocol.to_string(),
                })
            }
            _ => None,
        }
    }

    pub fn write_last_good(&self, pair: &ServerPair) {
        let _ = fs::create_dir_all(&self.paths.state_directory);
        let _ = fs::write(
            self.paths.last_good_file(),
            format!("{}{PAIR_SEPARATOR}{}\n", pair.nickname, pair.protocol),
        );
    }
}
