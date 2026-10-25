//! Test helpers shared by the app's test files. Each test file includes this module, so any one file uses
//! only part of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs, process};

use nasib_linux::engine_catalog::{Catalog, Entry, Region};

static NEXT_DIRECTORY_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A throwaway folder under the system temp folder, removed when dropped.
pub struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    pub fn new() -> TemporaryDirectory {
        let index = NEXT_DIRECTORY_INDEX.fetch_add(1, Ordering::SeqCst);
        let path = env::temp_dir().join(format!("nasib-linux-test-{}-{index}", process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TemporaryDirectory(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn entry(city: &str, nickname: &str, logged: bool) -> Entry {
    Entry {
        city: city.to_string(),
        nickname: nickname.to_string(),
        logged,
    }
}

/// The same small catalog the macOS LocationSelection tests use.
pub fn small_catalog() -> Catalog {
    Catalog {
        regions: vec![
            Region {
                name: "Turkey".to_string(),
                datacenters: vec![entry("Istanbul", "Lygos", false)],
            },
            Region {
                name: "United Arab Emirates".to_string(),
                datacenters: vec![entry("Dubai", "Khalifa", false)],
            },
            Region {
                name: "Canada East".to_string(),
                datacenters: vec![
                    entry("Toronto", "The 6", false),
                    entry("Toronto", "Comfort Zone", true),
                    entry("Toronto", "Peak", false),
                ],
            },
            Region {
                name: "Other".to_string(),
                datacenters: vec![entry("Toronto", "Drake", true)],
            },
        ],
    }
}
