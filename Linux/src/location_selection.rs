//! The locations checked in Settings, in the user's order. It starts from what the engine would search
//! for the saved settings, so opening Settings never changes the search by itself.

use nasib_engine::datacenter::Datacenter;

use crate::engine_catalog::{Catalog, Entry, Region};
use crate::engine_config::Configuration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationSelection {
    ordered: Vec<Entry>,
    catalog: Vec<Entry>,
}

/// What the engine searches for the configuration, chosen by the engine's own rules.
fn searched(configuration: &Configuration, catalog: &[Entry]) -> Vec<Entry> {
    let datacenters: Vec<Datacenter> = catalog
        .iter()
        .map(|entry| Datacenter {
            region: None,
            country_code: None,
            logged: entry.logged,
            city: entry.city.clone(),
            nickname: entry.nickname.clone(),
            addresses: Vec::new(),
        })
        .collect();
    configuration
        .select(&datacenters)
        .into_iter()
        .filter_map(|datacenter| {
            catalog
                .iter()
                .find(|entry| entry.nickname == datacenter.nickname)
                .cloned()
        })
        .collect()
}

fn contains_ignoring_case(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(&query.to_lowercase())
}

/// The entries whose title matches the search, in their order; all of them for a blank search.
pub fn filter_entries(entries: &[Entry], search: &str) -> Vec<Entry> {
    let query = search.trim();
    entries
        .iter()
        .filter(|entry| query.is_empty() || contains_ignoring_case(&entry.title(), query))
        .cloned()
        .collect()
}

/// The regions with only the datacenters that match the search. A region whose name matches keeps every
/// datacenter, and a region left empty is dropped.
pub fn filter_regions(catalog: &Catalog, search: &str) -> Vec<Region> {
    let query = search.trim();
    catalog
        .regions
        .iter()
        .filter_map(|region| {
            let datacenters = if contains_ignoring_case(&region.name, query) {
                region.datacenters.clone()
            } else {
                filter_entries(&region.datacenters, query)
            };
            (!datacenters.is_empty()).then(|| Region {
                name: region.name.clone(),
                datacenters,
            })
        })
        .collect()
}

impl LocationSelection {
    /// `None` when only cities are saved and the catalog couldn't be read: the cities can't be turned
    /// into locations then, and Settings must leave them as they are.
    pub fn new(saved: &Configuration, catalog: &Catalog) -> Option<LocationSelection> {
        let entries = catalog.entries();
        let ordered = match &saved.locations {
            // Without a catalog, the saved order stays editable under the nicknames alone.
            Some(locations) if entries.is_empty() => locations
                .iter()
                .map(|nickname| Entry {
                    city: String::new(),
                    nickname: nickname.clone(),
                    logged: false,
                })
                .collect(),
            None if entries.is_empty() => return None,
            _ => searched(saved, &entries),
        };
        Some(LocationSelection {
            ordered,
            catalog: entries,
        })
    }

    pub fn ordered(&self) -> &[Entry] {
        &self.ordered
    }

    pub fn nicknames(&self) -> Vec<String> {
        self.ordered
            .iter()
            .map(|entry| entry.nickname.clone())
            .collect()
    }

    /// An empty order would mean every datacenter to the engine, so Settings doesn't save one.
    pub fn can_save(&self) -> bool {
        !self.ordered.is_empty()
    }

    pub fn is_selected(&self, nickname: &str) -> bool {
        self.ordered.iter().any(|entry| entry.nickname == nickname)
    }

    pub fn are_all_selected(&self, nicknames: &[&str]) -> bool {
        nicknames.iter().all(|nickname| self.is_selected(nickname))
    }

    /// Appends the datacenter to the order, or removes it when it's already there.
    pub fn toggle(&mut self, nickname: &str) {
        if let Some(index) = self
            .ordered
            .iter()
            .position(|entry| entry.nickname == nickname)
        {
            self.ordered.remove(index);
        } else if let Some(entry) = self.catalog.iter().find(|entry| entry.nickname == nickname) {
            self.ordered.push(entry.clone());
        }
    }

    /// Appends the given datacenters that aren't checked yet, in the order given.
    pub fn select(&mut self, nicknames: &[&str]) {
        for nickname in nicknames {
            if !self.is_selected(nickname) {
                self.toggle(nickname);
            }
        }
    }

    pub fn deselect(&mut self, nicknames: &[&str]) {
        self.ordered
            .retain(|entry| !nicknames.contains(&entry.nickname.as_str()));
    }

    pub fn deselect_all(&mut self) {
        self.ordered.clear();
    }

    /// Moves the row at `from` so it ends up at index `to`.
    pub fn move_row(&mut self, from: usize, to: usize) {
        if from >= self.ordered.len() {
            return;
        }
        let entry = self.ordered.remove(from);
        self.ordered.insert(to.min(self.ordered.len()), entry);
    }

    /// Restores what an untouched install searches.
    pub fn reset_to_defaults(&mut self) {
        self.ordered = searched(&Configuration::default(), &self.catalog);
    }
}
