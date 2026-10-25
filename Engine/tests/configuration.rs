mod support;

use std::fs;
use std::path::PathBuf;

use nasib_engine::configuration::{Configuration, ALL_PROTOCOLS, DEFAULT_LOCATIONS};
use nasib_engine::datacenter::Datacenter;
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
            ..Configuration::default()
        };

        assert!(configuration.includes_city("Reykjavik"));
    }

    #[test]
    fn should_match_cities_without_regard_to_case() {
        let configuration = Configuration {
            cities: vec!["New York".to_string()],
            ..Configuration::default()
        };

        assert!(configuration.includes_city("new york"));
        assert!(!configuration.includes_city("York"));
    }
}

fn bundled(location_name: &str, region: Option<&str>) -> Datacenter {
    Datacenter {
        region: region.map(String::from),
        logged: false,
        ..Datacenter::from_location_name(location_name, vec![]).unwrap()
    }
}

/// A datacenter that is bundled and also named in the Windscribe log, as Comfort Zone is on a real Mac.
fn bundled_and_logged(location_name: &str, region: &str) -> Datacenter {
    Datacenter {
        logged: true,
        ..bundled(location_name, Some(region))
    }
}

fn nicknames(datacenters: &[Datacenter]) -> Vec<String> {
    datacenters
        .iter()
        .map(|datacenter| datacenter.nickname.clone())
        .collect()
}

fn loaded(text: &str) -> Configuration {
    let (_directory, path) = written_configuration(text);
    Configuration::load(&path)
}

mod locations {
    use super::*;

    #[test]
    fn should_default_to_carrying_over_the_default_cities_with_ranking_on() {
        let configuration = Configuration::default();

        assert_eq!(configuration.locations, None);
        assert_eq!(configuration.cities.len(), 16);
        assert!(configuration.rank_by_success);
    }

    #[test]
    fn should_read_saved_locations_and_ranking() {
        let configuration = loaded(
            r#"{ "locations": ["The 6", "Khalifa"], "rankBySuccess": false, "protocolOrder": ["tcp"] }"#,
        );

        assert_eq!(
            configuration.locations,
            Some(vec!["The 6".to_string(), "Khalifa".to_string()])
        );
        assert!(!configuration.rank_by_success);
        assert_eq!(configuration.protocol_order, vec!["tcp".to_string()]);
    }

    #[test]
    fn should_rank_by_success_when_the_key_is_missing() {
        assert!(loaded(r#"{ "cities": ["Dubai"], "protocolOrder": ["tcp"] }"#).rank_by_success);
    }

    #[test]
    fn should_leave_locations_unset_for_a_file_from_before_locations() {
        assert_eq!(
            loaded(r#"{ "cities": ["Dubai"], "protocolOrder": ["tcp"] }"#).locations,
            None
        );
    }
}

mod select {
    use super::*;

    fn catalog() -> Vec<Datacenter> {
        vec![
            bundled("Dubai - Khalifa", Some("United Arab Emirates")),
            bundled("Toronto - The 6", Some("Canada East")),
            bundled("Toronto - Comfort Zone", Some("Canada East")),
            bundled("Istanbul - Lygos", Some("Turkey")),
            Datacenter::from_location_name("Toronto - Drake", vec![]).unwrap(),
        ]
    }

    #[test]
    fn should_keep_todays_list_when_the_log_adds_a_bundled_datacenter() {
        let mut everything: Vec<Datacenter> = nasib_engine::catalog::bundled();
        let comfort_zone = everything
            .iter_mut()
            .find(|datacenter| datacenter.nickname == "Comfort Zone")
            .unwrap();
        comfort_zone.logged = true;

        let selected = nicknames(&Configuration::default().select(&everything));

        let mut expected = DEFAULT_LOCATIONS.map(String::from).to_vec();
        expected.push("Comfort Zone".to_string());
        assert_eq!(selected, expected);
    }

    #[test]
    fn should_append_every_logged_datacenter_in_saved_cities() {
        let configuration = loaded(r#"{ "cities": ["Toronto"], "protocolOrder": ["tcp"] }"#);
        let mut catalog = catalog();
        catalog[2] = bundled_and_logged("Toronto - Comfort Zone", "Canada East");

        assert_eq!(
            nicknames(&configuration.select(&catalog)),
            ["The 6", "Comfort Zone", "Drake"]
        );
    }

    #[test]
    fn should_select_todays_twenty_with_the_default_cities() {
        let everything: Vec<Datacenter> = nasib_engine::catalog::bundled();
        let configuration = Configuration {
            locations: None,
            ..Configuration::default()
        };

        assert_eq!(
            nicknames(&configuration.select(&everything)),
            DEFAULT_LOCATIONS.map(String::from).to_vec()
        );
    }

    #[test]
    fn should_select_default_locations_in_saved_cities_when_locations_is_missing() {
        let configuration =
            loaded(r#"{ "cities": ["Toronto", "Dubai"], "protocolOrder": ["tcp"] }"#);

        let selected = nicknames(&configuration.select(&catalog()));

        assert_eq!(&selected[..2], ["Khalifa", "The 6"]);
    }

    #[test]
    fn should_append_log_only_datacenters_in_saved_cities() {
        let configuration = loaded(r#"{ "cities": ["Toronto"], "protocolOrder": ["tcp"] }"#);

        assert_eq!(
            nicknames(&configuration.select(&catalog())),
            ["The 6", "Drake"]
        );
    }

    #[test]
    fn should_follow_the_saved_locations_order() {
        let configuration = loaded(
            r#"{ "locations": ["Lygos", "Comfort Zone", "Khalifa"], "protocolOrder": ["tcp"] }"#,
        );

        assert_eq!(
            nicknames(&configuration.select(&catalog())),
            ["Lygos", "Comfort Zone", "Khalifa"]
        );
    }

    #[test]
    fn should_skip_unknown_and_repeated_locations() {
        let configuration = loaded(
            r#"{ "locations": ["Retired", "Lygos", "Lygos", "Khalifa"], "protocolOrder": ["tcp"] }"#,
        );

        assert_eq!(
            nicknames(&configuration.select(&catalog())),
            ["Lygos", "Khalifa"]
        );
    }

    #[test]
    fn should_select_every_datacenter_when_locations_is_empty() {
        let configuration = loaded(r#"{ "locations": [], "protocolOrder": ["tcp"] }"#);

        assert_eq!(
            nicknames(&configuration.select(&catalog())),
            nicknames(&catalog())
        );
    }
}
