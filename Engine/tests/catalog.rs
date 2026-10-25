mod support;

use std::collections::HashSet;
use std::fs;

use nasib_engine::catalog;
use nasib_engine::configuration::{Configuration, DEFAULT_CITIES};
use nasib_engine::datacenter::Datacenter;
use support::TemporaryDirectory;

fn datacenter(location_name: &str, address: &str) -> Datacenter {
    Datacenter::from_location_name(location_name, vec![address.to_string()]).unwrap()
}

fn seed() -> Vec<Datacenter> {
    vec![
        datacenter("Istanbul - Lygos", "203.0.113.1"),
        datacenter("Amsterdam - Stolen Cheese", "203.0.113.2"),
    ]
}

fn every_city() -> Configuration {
    Configuration {
        cities: vec![],
        protocol_order: vec![],
    }
}

fn nicknames(datacenters: &[Datacenter]) -> Vec<&str> {
    datacenters
        .iter()
        .map(|datacenter| datacenter.nickname.as_str())
        .collect()
}

mod merge {
    use super::*;

    #[test]
    fn should_add_logged_addresses_to_a_known_nickname() {
        let logged = vec![datacenter("Amsterdam - Stolen Cheese", "203.0.113.9")];

        let merged = catalog::merge(seed(), logged, &every_city());

        assert_eq!(nicknames(&merged), ["Lygos", "Stolen Cheese"]);
        assert_eq!(merged[1].addresses, vec!["203.0.113.2", "203.0.113.9"]);
    }

    #[test]
    fn should_append_nicknames_only_the_log_knows() {
        let logged = vec![datacenter("Oslo - Fjord", "203.0.113.7")];

        let merged = catalog::merge(seed(), logged, &every_city());

        assert_eq!(nicknames(&merged), ["Lygos", "Stolen Cheese", "Fjord"]);
    }

    #[test]
    fn should_keep_only_the_chosen_cities() {
        let configuration = Configuration {
            cities: vec!["amsterdam".to_string()],
            protocol_order: vec![],
        };

        let merged = catalog::merge(seed(), vec![], &configuration);

        assert_eq!(nicknames(&merged), ["Stolen Cheese"]);
    }
}

mod seed_datacenters {
    use super::*;

    #[test]
    fn should_load_every_bundled_datacenter_with_its_addresses() {
        let datacenters = catalog::seed();

        assert_eq!(datacenters.len(), 20);
        assert!(datacenters
            .iter()
            .all(|datacenter| !datacenter.addresses.is_empty()));
        let unique: HashSet<&str> = nicknames(&datacenters).into_iter().collect();
        assert_eq!(unique.len(), datacenters.len());
    }

    #[test]
    fn should_cover_every_default_city() {
        let cities: HashSet<String> = catalog::seed()
            .into_iter()
            .map(|datacenter| datacenter.city)
            .collect();

        for city in DEFAULT_CITIES {
            assert!(cities.contains(city), "{city}");
        }
    }

    #[test]
    fn should_split_nicknames_that_contain_spaces_correctly() {
        let datacenter = catalog::seed()
            .into_iter()
            .find(|datacenter| datacenter.nickname == "Stolen Cheese")
            .unwrap();

        assert_eq!(datacenter.city, "Amsterdam");
        assert_eq!(datacenter.addresses, vec!["161.129.60.66", "161.129.60.34"]);
    }
}

mod load {
    use super::*;

    #[test]
    fn should_merge_the_windscribe_log_over_the_seed() {
        let directory = TemporaryDirectory::new();
        fs::write(
            directory.path().join("client.log"),
            "{\"msg\": \"MutableLocationInfo created:  \\\"Amsterdam - Stolen Cheese\\\" \\\"203.0.113.5; \\\" ; Selected node: 1\"}\n",
        )
        .unwrap();

        let datacenters = catalog::load(&every_city(), directory.path());

        let amsterdam = datacenters
            .iter()
            .find(|datacenter| datacenter.nickname == "Stolen Cheese")
            .unwrap();
        assert_eq!(
            amsterdam.addresses,
            vec!["161.129.60.66", "161.129.60.34", "203.0.113.5"]
        );
    }
}
