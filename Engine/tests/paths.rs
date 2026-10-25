use std::collections::HashMap;
use std::path::PathBuf;

use nasib_engine::paths::Paths;

fn paths_with(variables: &[(&str, &str)]) -> Paths {
    let variables: HashMap<String, String> = variables
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    Paths::from_variables(|name| variables.get(name).cloned())
}

mod file_names {
    use super::*;

    #[test]
    fn should_keep_the_state_file_names_under_the_state_folder() {
        let paths = paths_with(&[("NASIB_STATE_DIR", "/state")]);

        assert_eq!(paths.log_file(), PathBuf::from("/state/log.csv"));
        assert_eq!(paths.last_good_file(), PathBuf::from("/state/last_good"));
        assert_eq!(
            paths.configuration_file(),
            PathBuf::from("/state/config.json")
        );
        assert_eq!(paths.lock_file(), PathBuf::from("/state/lock"));
    }
}

mod overrides {
    use super::*;

    #[test]
    fn should_prefer_environment_overrides() {
        let paths = paths_with(&[
            ("HOME", "/home/someone"),
            ("LOCALAPPDATA", "/local"),
            ("NASIB_STATE_DIR", "/state"),
            ("NASIB_WINDSCRIBE_LOG_DIR", "/logs"),
            ("NASIB_WINDSCRIBE_CLI", "/fake/windscribe-cli"),
        ]);

        assert_eq!(paths.state_directory, PathBuf::from("/state"));
        assert_eq!(paths.windscribe_log_directory, PathBuf::from("/logs"));
        assert_eq!(paths.windscribe_cli, PathBuf::from("/fake/windscribe-cli"));
    }
}

#[cfg(target_os = "macos")]
mod macos_defaults {
    use super::*;

    #[test]
    fn should_use_dot_ws_find_on_macos() {
        let paths = paths_with(&[("HOME", "/Users/someone")]);

        assert_eq!(
            paths.state_directory,
            PathBuf::from("/Users/someone/.ws-find")
        );
        assert_eq!(
            paths.windscribe_log_directory,
            PathBuf::from("/Users/someone/Library/Application Support/Windscribe/Windscribe2")
        );
        assert_eq!(
            paths.windscribe_cli,
            PathBuf::from("/usr/local/bin/windscribe-cli")
        );
    }
}

#[cfg(target_os = "linux")]
mod linux_defaults {
    use super::*;

    #[test]
    fn should_use_the_xdg_data_folder_on_linux() {
        let paths = paths_with(&[("HOME", "/home/someone"), ("XDG_DATA_HOME", "/data")]);

        assert_eq!(paths.state_directory, PathBuf::from("/data/nasib"));
        assert_eq!(
            paths.windscribe_log_directory,
            PathBuf::from("/data/Windscribe/Windscribe2")
        );
        assert_eq!(
            paths.windscribe_cli,
            PathBuf::from("/usr/bin/windscribe-cli")
        );
    }

    #[test]
    fn should_fall_back_to_dot_local_share_on_linux() {
        let paths = paths_with(&[("HOME", "/home/someone")]);

        assert_eq!(
            paths.state_directory,
            PathBuf::from("/home/someone/.local/share/nasib")
        );
    }
}

#[cfg(windows)]
mod windows_defaults {
    use super::*;

    #[test]
    fn should_live_in_the_local_application_data_folder() {
        let paths = paths_with(&[
            ("LOCALAPPDATA", r"C:\Users\someone\AppData\Local"),
            ("ProgramFiles", r"C:\Program Files"),
        ]);

        assert_eq!(
            paths.state_directory,
            PathBuf::from(r"C:\Users\someone\AppData\Local\Nasib")
        );
        assert_eq!(
            paths.windscribe_log_directory,
            PathBuf::from(r"C:\Users\someone\AppData\Local\Windscribe\Windscribe2")
        );
        assert_eq!(
            paths.windscribe_cli,
            PathBuf::from(r"C:\Program Files\Windscribe\windscribe-cli.exe")
        );
    }
}
