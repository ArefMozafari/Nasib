//! Which cities to search and which protocols to try, in order, as saved from the Settings tab in
//! `config.json`. Every app writes the same format; the engine only reads it.

use std::fs;
use std::path::Path;

use serde::Deserialize;

/// Every protocol the engine knows, in the order it tries them unless Settings says otherwise.
pub const ALL_PROTOCOLS: [&str; 6] = ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"];

/// The cities searched until the user changes them in Settings.
pub const DEFAULT_CITIES: [&str; 16] = [
    "Istanbul",
    "Dubai",
    "Sofia",
    "Stockholm",
    "Amsterdam",
    "Frankfurt",
    "Zurich",
    "Toronto",
    "Montreal",
    "Winnipeg",
    "New York",
    "Chicago",
    "Atlanta",
    "Bratislava",
    "Helsinki",
    "Brussels",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// An empty list means every city.
    pub cities: Vec<String>,
    pub protocol_order: Vec<String>,
}

/// The file as stored. Both keys are required; a file missing either one means the defaults.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredConfiguration {
    cities: Vec<String>,
    protocol_order: Vec<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            cities: DEFAULT_CITIES.map(String::from).to_vec(),
            protocol_order: ALL_PROTOCOLS.map(String::from).to_vec(),
        }
    }
}

impl Configuration {
    /// Keeps only protocols the engine knows, each once, in the order given.
    fn known_protocols(protocols: Vec<String>) -> Vec<String> {
        let mut known: Vec<String> = Vec::new();
        for protocol in protocols {
            if ALL_PROTOCOLS.contains(&protocol.as_str()) && !known.contains(&protocol) {
                known.push(protocol);
            }
        }
        known
    }

    /// Reads the saved file, or returns the defaults when it is missing or unreadable.
    pub fn load(path: &Path) -> Configuration {
        let stored = fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<StoredConfiguration>(&text).ok());
        match stored {
            Some(stored) => Configuration {
                cities: stored.cities,
                protocol_order: Configuration::known_protocols(stored.protocol_order),
            },
            None => Configuration::default(),
        }
    }

    /// True when the city is searched: every city is when the list is empty.
    pub fn includes_city(&self, city: &str) -> bool {
        self.cities.is_empty()
            || self
                .cities
                .iter()
                .any(|candidate| candidate.to_lowercase() == city.to_lowercase())
    }
}
