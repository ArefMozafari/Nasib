//! Where the engine keeps its files and finds Windscribe's. Each platform keeps the folder it has
//! always used, so nobody's history moves.

use std::env;
use std::path::PathBuf;

/// Test overrides. The apps never set them.
const STATE_DIRECTORY_VARIABLE: &str = "NASIB_STATE_DIR";
const WINDSCRIBE_LOG_DIRECTORY_VARIABLE: &str = "NASIB_WINDSCRIBE_LOG_DIR";
const WINDSCRIBE_CLI_VARIABLE: &str = "NASIB_WINDSCRIBE_CLI";

const LOG_FILE_NAME: &str = "log.csv";
const LAST_GOOD_FILE_NAME: &str = "last_good";
const CONFIGURATION_FILE_NAME: &str = "config.json";
/// On macOS this is where ws-find.sh made its lock directory, which the run lock clears if it is left over.
const LOCK_FILE_NAME: &str = "lock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub state_directory: PathBuf,
    pub windscribe_log_directory: PathBuf,
    pub windscribe_cli: PathBuf,
}

/// The platform defaults, before any override.
struct PlatformDefaults {
    state_directory: PathBuf,
    windscribe_log_directory: PathBuf,
    windscribe_cli: PathBuf,
}

#[cfg(target_os = "macos")]
fn platform_defaults(variable: &dyn Fn(&str) -> Option<String>) -> PlatformDefaults {
    let home = PathBuf::from(variable("HOME").unwrap_or_default());
    PlatformDefaults {
        state_directory: home.join(".ws-find"),
        windscribe_log_directory: home.join("Library/Application Support/Windscribe/Windscribe2"),
        windscribe_cli: PathBuf::from("/usr/local/bin/windscribe-cli"),
    }
}

#[cfg(windows)]
fn platform_defaults(variable: &dyn Fn(&str) -> Option<String>) -> PlatformDefaults {
    let local_application_data = PathBuf::from(variable("LOCALAPPDATA").unwrap_or_default());
    let program_files =
        PathBuf::from(variable("ProgramFiles").unwrap_or_else(|| r"C:\Program Files".to_string()));
    PlatformDefaults {
        state_directory: local_application_data.join("Nasib"),
        windscribe_log_directory: local_application_data
            .join("Windscribe")
            .join("Windscribe2"),
        windscribe_cli: program_files.join("Windscribe").join("windscribe-cli.exe"),
    }
}

/// The Linux locations are the expected ones; they are confirmed when the Linux app is built.
#[cfg(not(any(target_os = "macos", windows)))]
fn platform_defaults(variable: &dyn Fn(&str) -> Option<String>) -> PlatformDefaults {
    let home = PathBuf::from(variable("HOME").unwrap_or_default());
    let data_home = variable("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));
    PlatformDefaults {
        state_directory: data_home.join("nasib"),
        windscribe_log_directory: data_home.join("Windscribe/Windscribe2"),
        windscribe_cli: PathBuf::from("/usr/bin/windscribe-cli"),
    }
}

impl Paths {
    /// Resolves the paths from the given environment lookup, preferring the test overrides.
    pub fn from_variables(variable: impl Fn(&str) -> Option<String>) -> Paths {
        let defaults = platform_defaults(&variable);
        let overridden =
            |name: &str, default: PathBuf| variable(name).map(PathBuf::from).unwrap_or(default);
        Paths {
            state_directory: overridden(STATE_DIRECTORY_VARIABLE, defaults.state_directory),
            windscribe_log_directory: overridden(
                WINDSCRIBE_LOG_DIRECTORY_VARIABLE,
                defaults.windscribe_log_directory,
            ),
            windscribe_cli: overridden(WINDSCRIBE_CLI_VARIABLE, defaults.windscribe_cli),
        }
    }

    pub fn for_current_user() -> Paths {
        Paths::from_variables(|name| env::var(name).ok())
    }

    pub fn log_file(&self) -> PathBuf {
        self.state_directory.join(LOG_FILE_NAME)
    }

    pub fn last_good_file(&self) -> PathBuf {
        self.state_directory.join(LAST_GOOD_FILE_NAME)
    }

    pub fn configuration_file(&self) -> PathBuf {
        self.state_directory.join(CONFIGURATION_FILE_NAME)
    }

    pub fn lock_file(&self) -> PathBuf {
        self.state_directory.join(LOCK_FILE_NAME)
    }
}
