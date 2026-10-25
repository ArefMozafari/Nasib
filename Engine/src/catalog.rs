//! Every datacenter the engine knows: the bundled list of all Windscribe datacenters, with whatever the
//! Windscribe log has added. The bundled list comes from Windscribe's public server list through
//! `Engine/tools/update-datacenters`; it matters most on Windows, where Windscribe logs a datacenter only
//! once it connects to it.

use std::path::Path;

use serde::Deserialize;

use crate::client_log;
use crate::datacenter::Datacenter;

const BUNDLED_DATACENTERS: &str = include_str!("datacenters.json");

#[derive(Deserialize)]
struct BundledFile {
    datacenters: Vec<BundledDatacenter>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundledDatacenter {
    region: String,
    country_code: String,
    city: String,
    nickname: String,
    addresses: Vec<String>,
}

/// The bundled datacenters, in the server list's order, which groups them by region.
pub fn bundled() -> Vec<Datacenter> {
    let file: BundledFile = serde_json::from_str(BUNDLED_DATACENTERS)
        .expect("the bundled datacenter list is valid JSON");
    file.datacenters
        .into_iter()
        .map(|bundled| Datacenter {
            region: Some(bundled.region),
            country_code: Some(bundled.country_code),
            logged: false,
            city: bundled.city,
            nickname: bundled.nickname,
            addresses: bundled.addresses,
        })
        .collect()
}

/// Keeps the bundled order, adds logged addresses to known nicknames and marks them logged, and appends
/// nicknames only the log knows, without a region.
pub fn merge(bundled: Vec<Datacenter>, logged: Vec<Datacenter>) -> Vec<Datacenter> {
    let mut merged = bundled;
    for datacenter in logged {
        match merged
            .iter_mut()
            .find(|existing| existing.nickname == datacenter.nickname)
        {
            Some(existing) => {
                *existing = Datacenter {
                    logged: true,
                    ..existing.with_addresses(&datacenter.addresses)
                }
            }
            None => merged.push(datacenter),
        }
    }
    merged
}

/// The bundled list merged with the Windscribe log in the given folder.
pub fn load(log_directory: &Path) -> Vec<Datacenter> {
    merge(bundled(), client_log::read_directory(log_directory))
}
