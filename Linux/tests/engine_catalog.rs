mod support;

use nasib_linux::engine_catalog::Catalog;
use support::{entry, small_catalog};

mod decode {
    use super::*;

    #[test]
    fn should_decode_the_catalog_line() {
        let line = r#"{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"The 6"}]},{"name":"Other","datacenters":[{"city":"Oslo","nickname":"Fjord"}]}]}"#;

        let catalog = Catalog::decode(line).unwrap();

        let names: Vec<&str> = catalog
            .regions
            .iter()
            .map(|region| region.name.as_str())
            .collect();
        assert_eq!(names, ["Canada East", "Other"]);
        assert_eq!(
            catalog.regions[0].datacenters[0],
            entry("Toronto", "The 6", false)
        );
    }

    #[test]
    fn should_read_which_datacenters_the_log_knows() {
        let line = r#"{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"Comfort Zone","logged":true},{"city":"Toronto","nickname":"The 6"}]}]}"#;

        let logged: Vec<bool> = Catalog::decode(line)
            .unwrap()
            .entries()
            .iter()
            .map(|entry| entry.logged)
            .collect();

        assert_eq!(logged, [true, false]);
    }

    #[test]
    fn should_return_none_for_anything_else() {
        assert!(Catalog::decode("windscribe-cli missing: /usr/bin/windscribe-cli").is_none());
        assert!(Catalog::decode("").is_none());
    }
}

mod entries {
    use super::*;

    #[test]
    fn should_list_every_entry_in_region_order() {
        let nicknames: Vec<String> = small_catalog()
            .entries()
            .into_iter()
            .map(|entry| entry.nickname)
            .collect();

        assert_eq!(
            nicknames,
            ["Lygos", "Khalifa", "The 6", "Comfort Zone", "Peak", "Drake"]
        );
    }

    #[test]
    fn should_title_entries_as_windscribe_shows_them() {
        assert_eq!(entry("Toronto", "The 6", false).title(), "Toronto - The 6");
        assert_eq!(entry("", "The 6", false).title(), "The 6");
    }
}
