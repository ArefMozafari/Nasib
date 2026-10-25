mod support;

use std::collections::HashSet;
use std::fs;

use nasib_engine::catalog;
use nasib_engine::configuration::DEFAULT_LOCATIONS;
use nasib_engine::datacenter::Datacenter;
use support::TemporaryDirectory;

const BUNDLED_MINIMUM: usize = 200;

fn datacenter(location_name: &str, address: &str) -> Datacenter {
    Datacenter::from_location_name(location_name, vec![address.to_string()]).unwrap()
}

fn bundled_pair() -> Vec<Datacenter> {
    vec![
        Datacenter {
            region: Some("Turkey".to_string()),
            country_code: Some("TR".to_string()),
            ..datacenter("Istanbul - Lygos", "203.0.113.1")
        },
        Datacenter {
            region: Some("Netherlands".to_string()),
            country_code: Some("NL".to_string()),
            ..datacenter("Amsterdam - Stolen Cheese", "203.0.113.2")
        },
    ]
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
    fn should_keep_the_region_when_the_log_adds_addresses() {
        let logged = vec![datacenter("Amsterdam - Stolen Cheese", "203.0.113.9")];

        let merged = catalog::merge(bundled_pair(), logged);

        assert_eq!(nicknames(&merged), ["Lygos", "Stolen Cheese"]);
        assert_eq!(merged[1].addresses, vec!["203.0.113.2", "203.0.113.9"]);
        assert_eq!(merged[1].region.as_deref(), Some("Netherlands"));
    }

    #[test]
    fn should_add_log_only_datacenters_without_a_region() {
        let logged = vec![datacenter("Oslo - Fjord", "203.0.113.7")];

        let merged = catalog::merge(bundled_pair(), logged);

        assert_eq!(nicknames(&merged), ["Lygos", "Stolen Cheese", "Fjord"]);
        assert_eq!(merged[2].region, None);
    }
}

mod bundled {
    use super::*;

    #[test]
    fn should_load_every_bundled_datacenter_with_a_region() {
        let datacenters = catalog::bundled();

        assert!(
            datacenters.len() >= BUNDLED_MINIMUM,
            "{}",
            datacenters.len()
        );
        // A datacenter may have no servers in the list for a while; it stays listed, and the log can add them.
        assert!(datacenters
            .iter()
            .all(|datacenter| datacenter.region.is_some() && datacenter.country_code.is_some()));
        let with_addresses = datacenters
            .iter()
            .filter(|datacenter| !datacenter.addresses.is_empty())
            .count();
        assert!(
            with_addresses * 10 >= datacenters.len() * 9,
            "{with_addresses}"
        );
        let unique: HashSet<&str> = nicknames(&datacenters).into_iter().collect();
        assert_eq!(unique.len(), datacenters.len());
    }

    #[test]
    fn should_include_every_default_location() {
        let datacenters = catalog::bundled();
        let bundled: HashSet<&str> = nicknames(&datacenters).into_iter().collect();

        for nickname in DEFAULT_LOCATIONS {
            assert!(bundled.contains(nickname), "{nickname}");
        }
    }

    #[test]
    fn should_split_nicknames_that_contain_spaces_correctly() {
        let datacenter = catalog::bundled()
            .into_iter()
            .find(|datacenter| datacenter.nickname == "Stolen Cheese")
            .unwrap();

        assert_eq!(datacenter.city, "Amsterdam");
        assert_eq!(datacenter.region.as_deref(), Some("Netherlands"));
    }
}

mod load {
    use super::*;

    #[test]
    fn should_merge_the_windscribe_log_over_the_bundled_list() {
        let directory = TemporaryDirectory::new();
        fs::write(
            directory.path().join("client.log"),
            "{\"msg\": \"MutableLocationInfo created:  \\\"Amsterdam - Stolen Cheese\\\" \\\"203.0.113.5; \\\" ; Selected node: 1\"}\n",
        )
        .unwrap();

        let datacenters = catalog::load(directory.path());

        let amsterdam = datacenters
            .iter()
            .find(|datacenter| datacenter.nickname == "Stolen Cheese")
            .unwrap();
        assert_eq!(
            amsterdam.addresses.last().map(String::as_str),
            Some("203.0.113.5")
        );
    }
}
