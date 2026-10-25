mod support;

use std::fs;
use std::path::Path;

use nasib_linux::login_item::{autostart_file, is_enabled, set_enabled};
use support::TemporaryDirectory;

const EXECUTABLE: &str = "/usr/lib/nasib/nasib";

mod login_item {
    use super::*;

    #[test]
    fn should_write_an_autostart_entry_with_background_flag() {
        let directory = TemporaryDirectory::new();

        set_enabled(directory.path(), true, Path::new(EXECUTABLE)).unwrap();

        let entry = fs::read_to_string(autostart_file(directory.path())).unwrap();
        assert!(entry.starts_with("[Desktop Entry]\n"), "{entry}");
        assert!(
            entry.contains("\nExec=/usr/lib/nasib/nasib --background\n"),
            "{entry}"
        );
        assert!(entry.contains("\nType=Application\n"), "{entry}");
        assert!(is_enabled(directory.path()));
    }

    // A tarball install lives in a folder the session's PATH may not include, so the entry names it.
    #[test]
    fn should_quote_an_executable_path_with_spaces() {
        let directory = TemporaryDirectory::new();

        set_enabled(
            directory.path(),
            true,
            Path::new("/home/me/My Apps/nasib/nasib"),
        )
        .unwrap();

        let entry = fs::read_to_string(autostart_file(directory.path())).unwrap();
        assert!(
            entry.contains("\nExec=\"/home/me/My Apps/nasib/nasib\" --background\n"),
            "{entry}"
        );
    }

    #[test]
    fn should_remove_the_entry_when_turned_off() {
        let directory = TemporaryDirectory::new();
        set_enabled(directory.path(), true, Path::new(EXECUTABLE)).unwrap();

        set_enabled(directory.path(), false, Path::new(EXECUTABLE)).unwrap();

        assert!(!autostart_file(directory.path()).exists());
        assert!(!is_enabled(directory.path()));
    }

    #[test]
    fn should_turn_off_when_already_off() {
        let directory = TemporaryDirectory::new();

        assert!(set_enabled(directory.path(), false, Path::new(EXECUTABLE)).is_ok());
    }

    #[test]
    fn should_place_the_entry_in_the_autostart_folder() {
        let directory = TemporaryDirectory::new();

        assert_eq!(
            autostart_file(directory.path()),
            directory.path().join("autostart").join("nasib.desktop")
        );
    }
}
