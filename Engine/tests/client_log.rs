mod support;

use std::fs;

use nasib_engine::client_log;
use support::TemporaryDirectory;

const LOCATION_LINE: &str = r#"{"tm": "2026-10-22 14:40:24.882", "lvl": "debug", "mod": "basic", "msg": "MutableLocationInfo created:  \"Amsterdam - Stolen Cheese\" \"203.0.113.1; 203.0.113.2; \" ; Selected node: 1"}"#;

mod parse_line {
    use super::*;

    #[test]
    fn should_read_the_city_nickname_and_addresses_from_a_json_line() {
        let datacenter = client_log::parse_line(LOCATION_LINE).unwrap();

        assert_eq!(datacenter.city, "Amsterdam");
        assert_eq!(datacenter.nickname, "Stolen Cheese");
        assert_eq!(datacenter.addresses, vec!["203.0.113.1", "203.0.113.2"]);
    }

    #[test]
    fn should_read_a_plain_text_line_without_the_json_wrapper() {
        let datacenter = client_log::parse_line(
            r#"MutableLocationInfo created:  "New York - Inside Job" "203.0.113.9; " "#,
        )
        .unwrap();

        assert_eq!(datacenter.city, "New York");
        assert_eq!(datacenter.nickname, "Inside Job");
        assert_eq!(datacenter.addresses, vec!["203.0.113.9"]);
    }

    #[test]
    fn should_read_a_location_with_no_addresses_yet() {
        let datacenter =
            client_log::parse_line(r#"MutableLocationInfo created:  "Dubai - Khalifa" "" "#)
                .unwrap();

        assert!(datacenter.addresses.is_empty());
    }

    #[test]
    fn should_ignore_lines_that_are_not_location_lines() {
        let lines = [
            r#"{"tm": "2026-10-22 14:40:24.882", "lvl": "debug", "mod": "user", "msg": "Location selected: \"131Amsterdam - Stolen Cheese\""}"#,
            r#"{"tm": "2026-10-22 14:40:24.882", "lvl": "info", "mod": "basic"}"#,
            r#"{"tm": "2026-10-22"#,
            "",
        ];
        for line in lines {
            assert_eq!(client_log::parse_line(line), None, "{line}");
        }
    }
}

mod read_directory {
    use super::*;

    #[test]
    fn should_merge_every_address_seen_per_nickname_across_both_files() {
        let directory = TemporaryDirectory::new();
        fs::write(
            directory.path().join("client.1.log"),
            format!("{LOCATION_LINE}\n"),
        )
        .unwrap();
        fs::write(
            directory.path().join("client.log"),
            format!(
                "{}\n{{\"msg\": \"something else\"}}\n",
                LOCATION_LINE.replace("203.0.113.2", "203.0.113.3")
            ),
        )
        .unwrap();

        let datacenters = client_log::read_directory(directory.path());

        assert_eq!(datacenters.len(), 1);
        assert_eq!(
            datacenters[0].addresses,
            vec!["203.0.113.1", "203.0.113.2", "203.0.113.3"]
        );
    }

    #[test]
    fn should_return_nothing_when_the_folder_has_no_logs() {
        let directory = TemporaryDirectory::new();

        assert!(client_log::read_directory(directory.path()).is_empty());
    }

    #[test]
    fn should_return_nothing_when_the_folder_is_missing() {
        let directory = TemporaryDirectory::new();

        assert!(client_log::read_directory(&directory.path().join("missing")).is_empty());
    }

    #[test]
    fn should_skip_lines_that_are_not_valid_text() {
        let directory = TemporaryDirectory::new();
        let mut bytes = vec![0xff, 0xfe, b'\n'];
        bytes.extend_from_slice(LOCATION_LINE.as_bytes());
        fs::write(directory.path().join("client.log"), bytes).unwrap();

        assert_eq!(client_log::read_directory(directory.path()).len(), 1);
    }
}
