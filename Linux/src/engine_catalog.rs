//! Every datacenter the engine knows, grouped by Windscribe's region names, as `nasib-engine --catalog`
//! prints it. Settings lists it so users can pick and order their locations.

use serde::Deserialize;

const TITLE_SEPARATOR: &str = " - ";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    pub city: String,
    pub nickname: String,
    /// True when the Windscribe log names it; settings from before locations carry these over. The
    /// engine leaves it out unless it's true.
    #[serde(default)]
    pub logged: bool,
}

impl Entry {
    /// The name Windscribe shows, such as "Toronto - The 6".
    pub fn title(&self) -> String {
        if self.city.is_empty() {
            self.nickname.clone()
        } else {
            format!("{}{TITLE_SEPARATOR}{}", self.city, self.nickname)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Region {
    pub name: String,
    pub datacenters: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct Catalog {
    pub regions: Vec<Region>,
}

impl Catalog {
    /// Reads the one line `--catalog` prints; `None` for anything else.
    pub fn decode(line: &str) -> Option<Catalog> {
        serde_json::from_str(line).ok()
    }

    /// Every datacenter, in region order.
    pub fn entries(&self) -> Vec<Entry> {
        self.regions
            .iter()
            .flat_map(|region| region.datacenters.iter().cloned())
            .collect()
    }
}
