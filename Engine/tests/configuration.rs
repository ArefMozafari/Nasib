mod support;

use std::fs;
use std::path::PathBuf;

use nasib_engine::configuration::{Configuration, ALL_PROTOCOLS};
use support::TemporaryDirectory;

const CONFIGURATION_FILE_NAME: &str = "config.json";

fn written_configuration(text: &str) -> (TemporaryDirectory, PathBuf) {
    let directory = TemporaryDirectory::new();
    let path = directory.path().join(CONFIGURATION_FILE_NAME);
    fs::write(&path, text).unwrap();
    (directory, path)
}

mod load {
    use super::*;

    #[test]
    fn should_return_the_defaults_when_the_file_is_missing() {
        let directory = TemporaryDirectory::new();

        let configuration = Configuration::load(&directory.path().join(CONFIGURATION_FILE_NAME));

        assert_eq!(configuration, Configuration::default());
    }

    #[test]
    fn should_return_the_defaults_when_the_file_is_not_valid_json() {
        let (_directory, path) = written_configuration("CITIES=('Dubai')");

        assert_eq!(Configuration::load(&path), Configuration::default());
    }

    #[test]
    fn should_return_the_defaults_when_a_key_is_missing() {
        let (_directory, path) = written_configuration(r#"{ "cities": ["Dubai"] }"#);

        assert_eq!(Configuration::load(&path), Configuration::default());
    }

    #[test]
    fn should_drop_unknown_and_duplicate_protocols() {
        let (_directory, path) = written_configuration(
            r#"{ "cities": ["Dubai"], "protocolOrder": ["tcp", "openvpn", "ikev2", "tcp"] }"#,
        );

        let configuration = Configuration::load(&path);

        assert_eq!(configuration.cities, vec!["Dubai".to_string()]);
        assert_eq!(
            configuration.protocol_order,
            vec!["tcp".to_string(), "ikev2".to_string()]
        );
    }

    #[test]
    fn should_keep_an_empty_protocol_order() {
        let (_directory, path) = written_configuration(r#"{ "cities": [], "protocolOrder": [] }"#);

        let configuration = Configuration::load(&path);

        assert!(configuration.cities.is_empty());
        assert!(configuration.protocol_order.is_empty());
    }

    #[test]
    fn should_keep_cities_with_quotes() {
        let (_directory, path) = written_configuration(
            r#"{ "cities": ["Xi'an", "New York"], "protocolOrder": ["udp"] }"#,
        );

        assert_eq!(
            Configuration::load(&path).cities,
            vec!["Xi'an".to_string(), "New York".to_string()]
        );
    }
}

mod default_configuration {
    use super::*;

    #[test]
    fn should_try_every_protocol_in_the_ws_find_order() {
        assert_eq!(
            Configuration::default().protocol_order,
            ALL_PROTOCOLS.map(String::from).to_vec()
        );
        assert_eq!(
            ALL_PROTOCOLS,
            ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"]
        );
    }

    #[test]
    fn should_search_the_sixteen_default_cities() {
        assert_eq!(
            Configuration::default().cities,
            [
                "Istanbul",
                "Dubai",
                "Sofia",
                "Stockholm",
                "Amsterdam",
                "Frankfurt",
                "Zurich",
                "Toronto",
                "Montreal",
                "Winnipeg",
                "New York",
                "Chicago",
                "Atlanta",
                "Bratislava",
                "Helsinki",
                "Brussels",
            ]
            .map(String::from)
            .to_vec()
        );
    }
}

mod includes_city {
    use super::*;

    #[test]
    fn should_include_every_city_when_the_list_is_empty() {
        let configuration = Configuration {
            cities: vec![],
            protocol_order: vec![],
        };

        assert!(configuration.includes_city("Reykjavik"));
    }

    #[test]
    fn should_match_cities_without_regard_to_case() {
        let configuration = Configuration {
            cities: vec!["New York".to_string()],
            protocol_order: vec![],
        };

        assert!(configuration.includes_city("new york"));
        assert!(!configuration.includes_city("York"));
    }
}
