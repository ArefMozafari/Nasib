use nasib_engine::catalog_output;
use nasib_engine::datacenter::Datacenter;
use serde_json::{json, Value};

fn datacenter(location_name: &str, region: Option<&str>) -> Datacenter {
    Datacenter {
        region: region.map(String::from),
        logged: false,
        ..Datacenter::from_location_name(location_name, vec![]).unwrap()
    }
}

mod render {
    use super::*;

    #[test]
    fn should_mark_the_datacenters_the_log_knows() {
        let datacenters = vec![
            Datacenter {
                logged: true,
                ..datacenter("Toronto - Comfort Zone", Some("Canada East"))
            },
            datacenter("Toronto - The 6", Some("Canada East")),
        ];

        let rendered: Value = serde_json::from_str(&catalog_output::render(&datacenters)).unwrap();

        assert_eq!(
            rendered["regions"][0]["datacenters"],
            json!([
                { "city": "Toronto", "nickname": "Comfort Zone", "logged": true },
                { "city": "Toronto", "nickname": "The 6" }
            ])
        );
    }

    #[test]
    fn should_group_by_region_in_order() {
        let datacenters = vec![
            datacenter("Toronto - The 6", Some("Canada East")),
            datacenter("Dubai - Khalifa", Some("United Arab Emirates")),
            datacenter("Montreal - Bagel Poutine", Some("Canada East")),
        ];

        let rendered: Value = serde_json::from_str(&catalog_output::render(&datacenters)).unwrap();

        assert_eq!(
            rendered,
            json!({ "regions": [
                { "name": "Canada East", "datacenters": [
                    { "city": "Toronto", "nickname": "The 6" },
                    { "city": "Montreal", "nickname": "Bagel Poutine" }
                ]},
                { "name": "United Arab Emirates", "datacenters": [
                    { "city": "Dubai", "nickname": "Khalifa" }
                ]}
            ]})
        );
    }

    #[test]
    fn should_put_log_only_datacenters_under_other() {
        let datacenters = vec![
            datacenter("Oslo - Fjord", None),
            datacenter("Dubai - Khalifa", Some("United Arab Emirates")),
        ];

        let rendered: Value = serde_json::from_str(&catalog_output::render(&datacenters)).unwrap();

        assert_eq!(rendered["regions"][1]["name"], "Other");
        assert_eq!(
            rendered["regions"][1]["datacenters"][0]["nickname"],
            "Fjord"
        );
    }

    #[test]
    fn should_print_one_line() {
        let rendered =
            catalog_output::render(&[datacenter("Dubai - Khalifa", Some("United Arab Emirates"))]);

        assert!(!rendered.contains('\n'));
    }
}
