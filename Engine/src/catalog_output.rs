//! The catalog the apps' Settings read through `--catalog`: every datacenter, grouped by Windscribe's
//! region names, as one JSON line. It's part of the output contract in `Engine/README.md`.

use serde::Serialize;

use crate::datacenter::Datacenter;

/// Where datacenters known only from the Windscribe log go, since the log names no region.
const OTHER_REGION: &str = "Other";

#[derive(Serialize)]
struct Catalog<'a> {
    regions: Vec<Region<'a>>,
}

#[derive(Serialize)]
struct Region<'a> {
    name: &'a str,
    datacenters: Vec<Entry<'a>>,
}

#[derive(Serialize)]
struct Entry<'a> {
    city: &'a str,
    nickname: &'a str,
    /// Present only when the Windscribe log names it, so Settings can carry over old city settings the
    /// way the engine does.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    logged: bool,
}

/// Groups the datacenters by region in the order each region first appears, with log-only datacenters
/// in a final "Other" region.
pub fn render(datacenters: &[Datacenter]) -> String {
    let mut regions: Vec<Region> = Vec::new();
    let mut other: Vec<Entry> = Vec::new();
    for datacenter in datacenters {
        let entry = Entry {
            city: &datacenter.city,
            nickname: &datacenter.nickname,
            logged: datacenter.logged,
        };
        match datacenter.region.as_deref() {
            Some(name) => match regions.iter_mut().find(|region| region.name == name) {
                Some(region) => region.datacenters.push(entry),
                None => regions.push(Region {
                    name,
                    datacenters: vec![entry],
                }),
            },
            None => other.push(entry),
        }
    }
    if !other.is_empty() {
        regions.push(Region {
            name: OTHER_REGION,
            datacenters: other,
        });
    }
    serde_json::to_string(&Catalog { regions }).expect("the catalog serializes to JSON")
}
