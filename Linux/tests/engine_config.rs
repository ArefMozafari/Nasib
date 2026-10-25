mod support;

use std::fs;
use std::path::PathBuf;

use nasib_linux::engine_config::{save, save_problem, Configuration};
use support::TemporaryDirectory;

const CONFIGURATION_FILE_NAME: &str = "config.json";

fn configuration_file(directory: &TemporaryDirectory) -> PathBuf {
    directory.path().join(CONFIGURATION_FILE_NAME)
}

mod save_and_load {
    use super::*;

    #[test]
    fn should_round_trip_locations_and_ranking() {
        let directory = TemporaryDirectory::new();
        let configuration = Configuration {
            locations: Some(vec!["The 6".to_string(), "Khalifa".to_string()]),
            rank_by_success: false,
            ..Configuration::default()
        };

        save(&configuration, &configuration_file(&directory)).unwrap();

        assert_eq!(
            Configuration::load(&configuration_file(&directory)),
            configuration
        );
    }

    #[test]
    fn should_write_the_keys_the_engine_reads() {
        let directory = TemporaryDirectory::new();
        let configuration = Configuration {
            locations: Some(vec!["Khalifa".to_string()]),
            ..Configuration::default()
        };
        save(&configuration, &configuration_file(&directory)).unwrap();

        let text = fs::read_to_string(configuration_file(&directory)).unwrap();

        for key in [
            "\"cities\"",
            "\"protocolOrder\"",
            "\"locations\"",
            "\"rankBySuccess\"",
        ] {
            assert!(text.contains(key), "{key}");
        }
    }

    #[test]
    fn should_leave_locations_out_when_unset() {
        let directory = TemporaryDirectory::new();
        save(&Configuration::default(), &configuration_file(&directory)).unwrap();

        let text = fs::read_to_string(configuration_file(&directory)).unwrap();

        assert!(!text.contains("\"locations\""), "{text}");
    }

    #[test]
    fn should_create_the_missing_folder() {
        let directory = TemporaryDirectory::new();
        let file = directory.path().join("nasib").join(CONFIGURATION_FILE_NAME);

        save(&Configuration::default(), &file).unwrap();

        assert_eq!(Configuration::load(&file), Configuration::default());
    }
}

mod protocol_rows {
    use super::*;
    use nasib_linux::engine_config::protocol_rows;

    #[test]
    fn should_list_enabled_protocols_in_order_then_disabled_ones_in_default_order() {
        let configuration = Configuration {
            protocol_order: vec!["tcp".to_string(), "wireguard".to_string()],
            ..Configuration::default()
        };

        assert_eq!(
            protocol_rows(&configuration),
            [
                ("tcp".to_string(), true),
                ("wireguard".to_string(), true),
                ("ikev2".to_string(), false),
                ("stealth".to_string(), false),
                ("wstunnel".to_string(), false),
                ("udp".to_string(), false),
            ]
        );
    }
}

mod save_rules {
    use super::*;

    #[test]
    fn should_refuse_to_save_without_a_location() {
        assert_eq!(
            save_problem(Some(0), 3),
            Some("Check at least one location.")
        );
    }

    #[test]
    fn should_refuse_to_save_without_a_protocol() {
        assert_eq!(
            save_problem(Some(2), 0),
            Some("Check at least one protocol.")
        );
    }

    #[test]
    fn should_save_when_locations_are_not_loaded() {
        assert_eq!(save_problem(None, 1), None);
    }

    #[test]
    fn should_save_with_a_location_and_a_protocol() {
        assert_eq!(save_problem(Some(1), 1), None);
    }
}
