//! Which cities to search and which protocols to try, in order, as saved from the Settings tab in
//! `config.json`. Every app writes the same format; the engine only reads it.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::datacenter::Datacenter;

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

/// The datacenters searched until the user changes them in Settings: the 20 the engine searched before
/// every Windscribe datacenter was bundled, in the same order, so an untouched install searches as before.
pub const DEFAULT_LOCATIONS: [&str; 20] = [
    "Lygos",
    "Khalifa",
    "Vitoshka",
    "Nevski",
    "Fika",
    "Djurgarden",
    "Stolen Cheese",
    "Wurstchen",
    "Sausage Party",
    "Altstadt",
    "The 6",
    "Bagel Poutine",
    "The Forks",
    "Inside Job",
    "Green Mill",
    "Magic City",
    "Devin Castle",
    "Sauna",
    "Tram",
    "Guildhouse",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// The datacenters to search by nickname, in the user's order; an empty list means every one.
    /// `None` for a file saved before locations existed, whose cities then decide.
    pub locations: Option<Vec<String>>,
    /// Reorders the locations by recent success; off, the user's order is followed exactly.
    pub rank_by_success: bool,
    pub protocol_order: Vec<String>,
    /// The cities saved before locations existed; an empty list means every city.
    pub cities: Vec<String>,
}

/// The file as stored. `protocolOrder` is required, and so is one of `locations` and `cities`; a file
/// missing them means the defaults.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredConfiguration {
    locations: Option<Vec<String>>,
    rank_by_success: Option<bool>,
    protocol_order: Vec<String>,
    cities: Option<Vec<String>>,
}

impl Default for Configuration {
    /// No saved locations, so an untouched install carries over the default cities and searches exactly
    /// what it did before every datacenter was bundled.
    fn default() -> Self {
        Configuration {
            locations: None,
            rank_by_success: true,
            protocol_order: ALL_PROTOCOLS.map(String::from).to_vec(),
            cities: DEFAULT_CITIES.map(String::from).to_vec(),
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
            Some(stored) if stored.locations.is_some() || stored.cities.is_some() => {
                Configuration {
                    locations: stored.locations,
                    rank_by_success: stored.rank_by_success.unwrap_or(true),
                    protocol_order: Configuration::known_protocols(stored.protocol_order),
                    cities: stored
                        .cities
                        .unwrap_or_else(|| DEFAULT_CITIES.map(String::from).to_vec()),
                }
            }
            _ => Configuration::default(),
        }
    }

    /// The datacenters to search, in order. Saved locations are followed in the user's order, skipping
    /// unknown and repeated ones. Without saved locations, the engine searches what it did before every
    /// datacenter was bundled: the default locations in the saved cities, then every other datacenter the
    /// Windscribe log knows in those cities.
    pub fn select(&self, datacenters: &[Datacenter]) -> Vec<Datacenter> {
        let find = |nickname: &str| {
            datacenters
                .iter()
                .find(|datacenter| datacenter.nickname == nickname)
        };
        let mut selected: Vec<Datacenter> = Vec::new();
        let mut add = |datacenter: &Datacenter| {
            if !selected
                .iter()
                .any(|existing| existing.nickname == datacenter.nickname)
            {
                selected.push(datacenter.clone());
            }
        };
        match &self.locations {
            Some(locations) if locations.is_empty() => datacenters.iter().for_each(&mut add),
            Some(locations) => locations
                .iter()
                .filter_map(|nickname| find(nickname))
                .for_each(&mut add),
            None => {
                DEFAULT_LOCATIONS
                    .iter()
                    .filter_map(|nickname| find(nickname))
                    .filter(|datacenter| self.includes_city(&datacenter.city))
                    .for_each(&mut add);
                datacenters
                    .iter()
                    .filter(|datacenter| datacenter.logged && self.includes_city(&datacenter.city))
                    .for_each(&mut add);
            }
        }
        selected
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
