mod support;

use nasib_linux::engine_catalog::Catalog;
use nasib_linux::engine_config::Configuration;
use nasib_linux::location_selection::LocationSelection;
use support::small_catalog;

fn saved(locations: Option<&[&str]>, cities: &[&str]) -> Configuration {
    Configuration {
        locations: locations.map(|names| names.iter().map(|name| name.to_string()).collect()),
        cities: cities.iter().map(|city| city.to_string()).collect(),
        ..Configuration::default()
    }
}

fn selection(locations: &[&str]) -> LocationSelection {
    LocationSelection::new(&saved(Some(locations), &[]), &small_catalog()).unwrap()
}

mod starting_point {
    use super::*;

    #[test]
    fn should_start_from_saved_locations_in_order() {
        assert_eq!(
            selection(&["The 6", "Lygos"]).nicknames(),
            ["The 6", "Lygos"]
        );
    }

    #[test]
    fn should_start_from_default_locations_then_logged_ones_in_saved_cities() {
        let selection =
            LocationSelection::new(&saved(None, &["Toronto", "Dubai"]), &small_catalog()).unwrap();

        assert_eq!(
            selection.nicknames(),
            ["Khalifa", "The 6", "Comfort Zone", "Drake"]
        );
    }

    #[test]
    fn should_start_from_every_location_when_the_saved_list_is_empty() {
        assert_eq!(
            selection(&[]).nicknames(),
            ["Lygos", "Khalifa", "The 6", "Comfort Zone", "Peak", "Drake"]
        );
    }

    #[test]
    fn should_keep_saved_locations_when_the_catalog_cannot_be_read() {
        let selection = LocationSelection::new(
            &saved(Some(&["The 6", "Lygos"]), &[]),
            &Catalog { regions: vec![] },
        )
        .unwrap();

        assert_eq!(selection.nicknames(), ["The 6", "Lygos"]);
    }

    // Without the catalog, the old cities can't be turned into locations, so Settings must not write any.
    #[test]
    fn should_not_start_without_a_catalog_when_only_cities_are_saved() {
        assert!(
            LocationSelection::new(&saved(None, &["Toronto"]), &Catalog { regions: vec![] })
                .is_none()
        );
    }
}

mod searching {
    use super::*;
    use nasib_linux::location_selection::{filter_entries, filter_regions};

    #[test]
    fn should_keep_every_entry_for_a_blank_query() {
        let selection = selection(&["The 6", "Lygos"]);

        assert_eq!(filter_entries(selection.ordered(), "  ").len(), 2);
    }

    #[test]
    fn should_match_titles_ignoring_case() {
        let selection = selection(&["The 6", "Lygos", "Khalifa"]);

        let nicknames: Vec<String> = filter_entries(selection.ordered(), "toronto")
            .into_iter()
            .map(|entry| entry.nickname)
            .collect();

        assert_eq!(nicknames, ["The 6"]);
    }

    #[test]
    fn should_keep_a_whole_region_whose_name_matches() {
        let regions = filter_regions(&small_catalog(), "canada");

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].datacenters.len(), 3);
    }

    #[test]
    fn should_keep_only_matching_datacenters_and_drop_empty_regions() {
        let regions = filter_regions(&small_catalog(), "peak");

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].name, "Canada East");
        assert_eq!(regions[0].datacenters[0].nickname, "Peak");
    }
}

mod editing {
    use super::*;

    #[test]
    fn should_append_when_toggled_on() {
        let mut selection = selection(&["The 6"]);
        selection.toggle("Peak");

        assert_eq!(selection.nicknames(), ["The 6", "Peak"]);
        assert!(selection.is_selected("Peak"));
    }

    #[test]
    fn should_remove_when_toggled_off() {
        let mut selection = selection(&["The 6", "Lygos"]);
        selection.toggle("The 6");

        assert_eq!(selection.nicknames(), ["Lygos"]);
        assert!(!selection.is_selected("The 6"));
    }

    #[test]
    fn should_move_a_row() {
        let mut selection = selection(&["The 6", "Lygos", "Khalifa"]);
        selection.move_row(2, 0);

        assert_eq!(selection.nicknames(), ["Khalifa", "The 6", "Lygos"]);
    }

    #[test]
    fn should_select_all_given_locations_after_the_order_in_their_order() {
        let mut selection = selection(&["Comfort Zone"]);
        selection.select(&["The 6", "Comfort Zone", "Peak"]);

        assert_eq!(selection.nicknames(), ["Comfort Zone", "The 6", "Peak"]);
        assert!(selection.are_all_selected(&["The 6", "Comfort Zone", "Peak"]));
    }

    #[test]
    fn should_deselect_only_the_given_locations() {
        let mut selection = selection(&["Lygos", "The 6", "Peak"]);
        selection.deselect(&["The 6", "Peak", "Drake"]);

        assert_eq!(selection.nicknames(), ["Lygos"]);
        assert!(!selection.are_all_selected(&["Lygos", "The 6"]));
    }

    #[test]
    fn should_deselect_everything() {
        let mut selection = selection(&["Lygos", "The 6"]);
        selection.deselect_all();

        assert!(selection.nicknames().is_empty());
        assert!(!selection.can_save());
    }

    #[test]
    fn should_reset_to_what_an_untouched_install_searches() {
        let mut selection = selection(&["Peak"]);
        selection.reset_to_defaults();

        assert_eq!(
            selection.nicknames(),
            ["Lygos", "Khalifa", "The 6", "Comfort Zone", "Drake"]
        );
    }

    #[test]
    fn should_not_save_an_empty_order() {
        let mut selection = selection(&["The 6"]);
        assert!(selection.can_save());

        selection.toggle("The 6");

        assert!(!selection.can_save());
    }
}
