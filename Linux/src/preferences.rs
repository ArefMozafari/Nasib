//! App-only choices that the engine doesn't read, kept next to the engine's files in the same format the
//! Windows app uses.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "preferences.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPreferences {
    auto_reconnect: Option<bool>,
    notified_update_version: Option<String>,
}

pub struct Preferences {
    path: PathBuf,
    auto_reconnect: bool,
    notified_update_version: Option<String>,
}

impl Preferences {
    /// Reads the saved choices from the state folder, or the defaults when the file is missing or broken.
    pub fn load(directory: &Path) -> Preferences {
        let path = directory.join(FILE_NAME);
        let stored = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<StoredPreferences>(&text).ok());
        Preferences {
            auto_reconnect: stored
                .as_ref()
                .and_then(|stored| stored.auto_reconnect)
                .unwrap_or(true),
            notified_update_version: stored.and_then(|stored| stored.notified_update_version),
            path,
        }
    }

    /// On unless the user turned it off.
    pub fn auto_reconnect(&self) -> bool {
        self.auto_reconnect
    }

    /// The newer version the user was already told about, so a restart doesn't notify again.
    pub fn notified_update_version(&self) -> Option<&str> {
        self.notified_update_version.as_deref()
    }

    pub fn set_auto_reconnect(&mut self, enabled: bool) {
        self.auto_reconnect = enabled;
        self.save();
    }

    pub fn set_notified_update_version(&mut self, version: &str) {
        self.notified_update_version = Some(version.to_string());
        self.save();
    }

    /// Best effort: when saving fails, the choices still hold for this run and fall back to the defaults
    /// next time.
    fn save(&self) {
        let stored = StoredPreferences {
            auto_reconnect: Some(self.auto_reconnect),
            notified_update_version: self.notified_update_version.clone(),
        };
        if let Some(folder) = self.path.parent() {
            let _ = fs::create_dir_all(folder);
        }
        if let Ok(text) = serde_json::to_string_pretty(&stored) {
            let _ = fs::write(&self.path, text);
        }
    }
}
