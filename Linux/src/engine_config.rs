//! Writes `config.json` from the Settings tab. The engine owns the format and reads it, so loading and
//! the defaults come from its `Configuration`.

use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;

pub use nasib_engine::configuration::{Configuration, ALL_PROTOCOLS};

use crate::text::{NO_LOCATION_PROBLEM, NO_PROTOCOL_PROBLEM};

/// The file as the engine reads it. `locations` is left out when unset, so the engine keeps carrying over
/// the saved cities.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredConfiguration<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    locations: Option<&'a [String]>,
    rank_by_success: bool,
    protocol_order: &'a [String],
    cities: &'a [String],
}

/// Saves the configuration, creating its folder when missing.
pub fn save(configuration: &Configuration, path: &Path) -> io::Result<()> {
    let stored = StoredConfiguration {
        locations: configuration.locations.as_deref(),
        rank_by_success: configuration.rank_by_success,
        protocol_order: &configuration.protocol_order,
        cities: &configuration.cities,
    };
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let text = serde_json::to_string_pretty(&stored).map_err(io::Error::other)?;
    fs::write(path, text)
}

/// The rows of the protocol list, as name and whether it's tried: the enabled protocols first in their
/// saved order, then the disabled ones in the default order.
pub fn protocol_rows(configuration: &Configuration) -> Vec<(String, bool)> {
    let enabled = configuration
        .protocol_order
        .iter()
        .map(|protocol| (protocol.clone(), true));
    let disabled = ALL_PROTOCOLS
        .iter()
        .filter(|protocol| {
            !configuration
                .protocol_order
                .iter()
                .any(|enabled| enabled == *protocol)
        })
        .map(|protocol| (protocol.to_string(), false));
    enabled.chain(disabled).collect()
}

/// Why Settings can't save yet, or `None` when it can. `location_count` is `None` when the locations
/// couldn't be loaded, which leaves the saved ones as they are.
pub fn save_problem(location_count: Option<usize>, protocol_count: usize) -> Option<&'static str> {
    if location_count == Some(0) {
        return Some(NO_LOCATION_PROBLEM);
    }
    if protocol_count == 0 {
        return Some(NO_PROTOCOL_PROBLEM);
    }
    None
}
