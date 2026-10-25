//! Reads datacenters out of the Windscribe app's client log. Each line is one JSON object, and the lines
//! that matter carry a "msg" such as:
//! `MutableLocationInfo created:  "Amsterdam - Stolen Cheese" "203.0.113.1; 203.0.113.2; " ; Selected node: 1`

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::datacenter::Datacenter;

/// The log before the current one, then the current one; Windscribe rotates on every start.
const LOG_FILE_NAMES: [&str; 2] = ["client.1.log", "client.log"];
const MESSAGE_KEY: &str = "msg";
const LOCATION_MARKER: &str = "MutableLocationInfo created:";
const QUOTE: char = '"';
const ADDRESS_SEPARATOR: char = ';';

/// Returns the message of a JSON line, or the line itself when it is not JSON, which covers older
/// plain-text logs. A JSON line without a text message has nothing to offer.
fn message_of(line: &str) -> String {
    match serde_json::from_str::<Value>(line) {
        Ok(value) => value
            .get(MESSAGE_KEY)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        Err(_) => line.to_string(),
    }
}

/// Reads one double-quoted field after optional whitespace, returning it and the text after it.
/// The closing quote may be missing, as long as every character so far is allowed.
fn quoted_field(text: &str, is_allowed: impl Fn(char) -> bool) -> Option<(&str, &str)> {
    let text = text.trim_start().strip_prefix(QUOTE)?;
    let end = text
        .find(|character: char| character == QUOTE || !is_allowed(character))
        .unwrap_or(text.len());
    Some((
        &text[..end],
        text[end..].strip_prefix(QUOTE).unwrap_or(&text[end..]),
    ))
}

fn is_address_character(character: char) -> bool {
    character.is_ascii_digit()
        || character == '.'
        || character == ADDRESS_SEPARATOR
        || character.is_whitespace()
}

/// Returns one datacenter for a location line, or `None` for any other line.
pub fn parse_line(line: &str) -> Option<Datacenter> {
    let message = message_of(line);
    let (_, after_marker) = message.split_once(LOCATION_MARKER)?;
    if !after_marker.starts_with(char::is_whitespace) {
        return None;
    }
    let (name, rest) = quoted_field(after_marker, |_| true)?;
    let (address_text, _) = quoted_field(rest, is_address_character)?;
    let addresses = address_text
        .split(ADDRESS_SEPARATOR)
        .map(str::trim)
        .filter(|address| !address.is_empty())
        .map(String::from)
        .collect();
    Datacenter::from_location_name(name, addresses)
}

/// Reads both log files in the folder and merges every address seen per nickname, in the order first seen.
/// Missing or unreadable files are skipped.
pub fn read_directory(directory: &Path) -> Vec<Datacenter> {
    let mut merged: Vec<Datacenter> = Vec::new();
    for file_name in LOG_FILE_NAMES {
        let Ok(bytes) = fs::read(directory.join(file_name)) else {
            continue;
        };
        for datacenter in String::from_utf8_lossy(&bytes)
            .lines()
            .filter_map(parse_line)
        {
            match merged
                .iter_mut()
                .find(|existing| existing.nickname == datacenter.nickname)
            {
                Some(existing) => *existing = existing.with_addresses(&datacenter.addresses),
                None => merged.push(datacenter),
            }
        }
    }
    merged
}
