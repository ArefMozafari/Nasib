//! The datacenters to search: the bundled list with whatever the Windscribe log has added, limited to
//! the chosen cities. The bundled list matters on Windows, where Windscribe logs a datacenter only once
//! it connects to it.

use std::path::Path;

use serde::Deserialize;

use crate::client_log;
use crate::configuration::Configuration;
use crate::datacenter::Datacenter;

const SEED_DATACENTERS: &str = include_str!("seed_datacenters.json");

#[derive(Deserialize)]
struct SeedDatacenter {
    city: String,
    nickname: String,
    addresses: Vec<String>,
}

/// The bundled datacenters, in their file order.
pub fn seed() -> Vec<Datacenter> {
    let stored: Vec<SeedDatacenter> =
        serde_json::from_str(SEED_DATACENTERS).expect("the bundled seed list is valid JSON");
    stored
        .into_iter()
        .map(|seed| Datacenter {
            city: seed.city,
            nickname: seed.nickname,
            addresses: seed.addresses,
        })
        .collect()
}

/// Keeps the seed order, adds logged addresses to known nicknames, appends nicknames only the log
/// knows, and keeps the chosen cities.
pub fn merge(
    seed: Vec<Datacenter>,
    logged: Vec<Datacenter>,
    configuration: &Configuration,
) -> Vec<Datacenter> {
    let mut merged = seed;
    for datacenter in logged {
        match merged
            .iter_mut()
            .find(|existing| existing.nickname == datacenter.nickname)
        {
            Some(existing) => *existing = existing.with_addresses(&datacenter.addresses),
            None => merged.push(datacenter),
        }
    }
    merged.retain(|datacenter| configuration.includes_city(&datacenter.city));
    merged
}

pub fn load(configuration: &Configuration, log_directory: &Path) -> Vec<Datacenter> {
    merge(
        seed(),
        client_log::read_directory(log_directory),
        configuration,
    )
}
