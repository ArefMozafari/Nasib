mod support;

use std::fs;

use nasib_linux::preferences::Preferences;
use support::TemporaryDirectory;

mod load {
    use super::*;

    #[test]
    fn should_turn_auto_reconnect_on_by_default() {
        let preferences = Preferences::load(TemporaryDirectory::new().path());

        assert!(preferences.auto_reconnect());
        assert_eq!(preferences.notified_update_version(), None);
    }

    #[test]
    fn should_read_the_windows_format() {
        let directory = TemporaryDirectory::new();
        fs::write(
            directory.path().join("preferences.json"),
            r#"{ "autoReconnect": false, "notifiedUpdateVersion": "0.4.0" }"#,
        )
        .unwrap();

        let preferences = Preferences::load(directory.path());

        assert!(!preferences.auto_reconnect());
        assert_eq!(preferences.notified_update_version(), Some("0.4.0"));
    }

    #[test]
    fn should_fall_back_to_the_defaults_for_a_broken_file() {
        let directory = TemporaryDirectory::new();
        fs::write(directory.path().join("preferences.json"), "not json").unwrap();

        assert!(Preferences::load(directory.path()).auto_reconnect());
    }
}

mod save {
    use super::*;

    #[test]
    fn should_keep_changes_across_loads() {
        let directory = TemporaryDirectory::new();
        let mut preferences = Preferences::load(directory.path());

        preferences.set_auto_reconnect(false);
        preferences.set_notified_update_version("0.5.0");

        let reloaded = Preferences::load(directory.path());
        assert!(!reloaded.auto_reconnect());
        assert_eq!(reloaded.notified_update_version(), Some("0.5.0"));
    }
}
