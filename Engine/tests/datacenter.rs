use nasib_engine::datacenter::Datacenter;

fn addresses(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

mod from_location_name {
    use super::*;

    #[test]
    fn should_split_the_city_from_the_nickname_at_the_first_separator() {
        let datacenter =
            Datacenter::from_location_name("New York - Inside - Job", addresses(&["203.0.113.1"]))
                .unwrap();

        assert_eq!(datacenter.city, "New York");
        assert_eq!(datacenter.nickname, "Inside - Job");
        assert_eq!(datacenter.addresses, addresses(&["203.0.113.1"]));
    }

    #[test]
    fn should_reject_names_without_a_city_and_a_nickname() {
        for location_name in ["Amsterdam", " - Stolen Cheese", "Amsterdam - "] {
            assert_eq!(
                Datacenter::from_location_name(location_name, vec![]),
                None,
                "{location_name}"
            );
        }
    }
}

mod location_name {
    use super::*;

    #[test]
    fn should_join_the_city_and_nickname_as_windscribe_shows_them() {
        let datacenter =
            Datacenter::from_location_name("Amsterdam - Stolen Cheese", vec![]).unwrap();

        assert_eq!(datacenter.location_name(), "Amsterdam - Stolen Cheese");
    }
}

mod with_addresses {
    use super::*;

    #[test]
    fn should_add_new_addresses_after_the_known_ones_without_duplicates() {
        let datacenter = Datacenter::from_location_name(
            "Dubai - Khalifa",
            addresses(&["203.0.113.1", "203.0.113.2"]),
        )
        .unwrap();

        let merged = datacenter.with_addresses(&addresses(&["203.0.113.2", "203.0.113.3"]));

        assert_eq!(
            merged.addresses,
            addresses(&["203.0.113.1", "203.0.113.2", "203.0.113.3"])
        );
    }
}
