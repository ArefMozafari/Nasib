mod support;

use std::fs;

use nasib_engine::attempt_log::{AttemptLog, AttemptRecord, Clock, ServerPair};
use nasib_engine::paths::Paths;
use support::TemporaryDirectory;

const FIXED_TIMESTAMP: &str = "2026-10-22T14:40:24";

struct FixedClock;

impl Clock for FixedClock {
    fn local_timestamp(&self) -> String {
        FIXED_TIMESTAMP.to_string()
    }
}

fn paths_in(directory: &TemporaryDirectory) -> Paths {
    let state_directory = directory.path().join("state");
    Paths::from_variables(|name| match name {
        "NASIB_STATE_DIR" => Some(state_directory.to_string_lossy().into_owned()),
        _ => None,
    })
}

fn attempt_log_in(directory: &TemporaryDirectory) -> AttemptLog {
    AttemptLog::new(paths_in(directory), Box::new(FixedClock))
}

fn record(timestamp: &str, nickname: &str, protocol: &str, succeeded: bool) -> AttemptRecord {
    AttemptRecord {
        timestamp: timestamp.to_string(),
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        succeeded,
    }
}

mod record_attempt {
    use super::*;

    #[test]
    fn should_append_lines_in_the_log_csv_format() {
        let directory = TemporaryDirectory::new();
        let attempt_log = attempt_log_in(&directory);

        attempt_log.record("Stolen Cheese", "wireguard", true);
        attempt_log.record("Khalifa", "ikev2", false);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).log_file()).unwrap(),
            "2026-10-22T14:40:24,Stolen Cheese,wireguard,ok\n2026-10-22T14:40:24,Khalifa,ikev2,fail\n"
        );
    }

    #[test]
    fn should_start_a_new_line_after_a_partial_last_line() {
        let directory = TemporaryDirectory::new();
        let paths = paths_in(&directory);
        fs::create_dir_all(&paths.state_directory).unwrap();
        fs::write(paths.log_file(), "2026-10-22T14:00:00,Lygos,tc").unwrap();

        attempt_log_in(&directory).record("Khalifa", "ikev2", true);

        assert_eq!(
            fs::read_to_string(paths.log_file()).unwrap(),
            "2026-10-22T14:00:00,Lygos,tc\n2026-10-22T14:40:24,Khalifa,ikev2,ok\n"
        );
    }
}

mod read_recent {
    use super::*;

    #[test]
    fn should_return_only_the_last_lines_and_skip_malformed_ones() {
        let directory = TemporaryDirectory::new();
        let paths = paths_in(&directory);
        fs::create_dir_all(&paths.state_directory).unwrap();
        fs::write(
            paths.log_file(),
            "2026-10-22T14:00:00,Lygos,tcp,ok\n2026-10-22T14:01:00,Khalifa,ikev2,fail\nnot a line\n2026-10-22T14:02:00,Fika,udp,ok\n",
        )
        .unwrap();

        let recent = attempt_log_in(&directory).read_recent(3);

        assert_eq!(
            recent,
            vec![
                record("2026-10-22T14:01:00", "Khalifa", "ikev2", false),
                record("2026-10-22T14:02:00", "Fika", "udp", true)
            ]
        );
    }

    #[test]
    fn should_return_nothing_before_the_first_attempt() {
        let directory = TemporaryDirectory::new();

        assert!(attempt_log_in(&directory).read_recent(300).is_empty());
    }
}

mod last_good {
    use super::*;

    #[test]
    fn should_round_trip_the_last_good_pair() {
        let directory = TemporaryDirectory::new();
        let attempt_log = attempt_log_in(&directory);
        let pair = ServerPair {
            nickname: "Stolen Cheese".to_string(),
            protocol: "wireguard".to_string(),
        };

        attempt_log.write_last_good(&pair);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).last_good_file()).unwrap(),
            "Stolen Cheese|wireguard\n"
        );
        assert_eq!(attempt_log.read_last_good(), Some(pair));
    }

    #[test]
    fn should_read_no_pair_from_a_missing_or_malformed_file() {
        for contents in [
            None,
            Some(""),
            Some("Khalifa"),
            Some("|ikev2"),
            Some("Khalifa|"),
            Some("a|b|c"),
        ] {
            let directory = TemporaryDirectory::new();
            let paths = paths_in(&directory);
            if let Some(contents) = contents {
                fs::create_dir_all(&paths.state_directory).unwrap();
                fs::write(paths.last_good_file(), contents).unwrap();
            }

            assert_eq!(
                attempt_log_in(&directory).read_last_good(),
                None,
                "{contents:?}"
            );
        }
    }
}
