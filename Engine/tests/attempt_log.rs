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

fn pair(nickname: &str, protocol: &str, port: Option<u16>) -> ServerPair {
    ServerPair {
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        port,
    }
}

fn record(
    timestamp: &str,
    nickname: &str,
    protocol: &str,
    port: Option<u16>,
    succeeded: bool,
) -> AttemptRecord {
    AttemptRecord {
        timestamp: timestamp.to_string(),
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        port,
        succeeded,
    }
}

fn write_log(directory: &TemporaryDirectory, contents: &str) {
    let paths = paths_in(directory);
    fs::create_dir_all(&paths.state_directory).unwrap();
    fs::write(paths.log_file(), contents).unwrap();
}

mod record_attempt {
    use super::*;

    #[test]
    fn should_append_the_port_as_a_fifth_column() {
        let directory = TemporaryDirectory::new();
        let attempt_log = attempt_log_in(&directory);

        attempt_log.record(&pair("Stolen Cheese", "wireguard", Some(443)), true);
        attempt_log.record(&pair("Khalifa", "ikev2", Some(500)), false);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).log_file()).unwrap(),
            "2026-10-22T14:40:24,Stolen Cheese,wireguard,ok,443\n2026-10-22T14:40:24,Khalifa,ikev2,fail,500\n"
        );
    }

    #[test]
    fn should_leave_the_port_column_empty_without_a_port() {
        let directory = TemporaryDirectory::new();

        attempt_log_in(&directory).record(&pair("Khalifa", "ikev2", None), true);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).log_file()).unwrap(),
            "2026-10-22T14:40:24,Khalifa,ikev2,ok,\n"
        );
    }

    #[test]
    fn should_start_a_new_line_after_a_partial_last_line() {
        let directory = TemporaryDirectory::new();
        write_log(&directory, "2026-10-22T14:00:00,Lygos,tc");

        attempt_log_in(&directory).record(&pair("Khalifa", "ikev2", Some(500)), true);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).log_file()).unwrap(),
            "2026-10-22T14:00:00,Lygos,tc\n2026-10-22T14:40:24,Khalifa,ikev2,ok,500\n"
        );
    }
}

mod read_recent {
    use super::*;

    #[test]
    fn should_return_only_the_last_lines_and_skip_malformed_ones() {
        let directory = TemporaryDirectory::new();
        write_log(
            &directory,
            "2026-10-22T14:00:00,Lygos,tcp,ok\n2026-10-22T14:01:00,Khalifa,ikev2,fail\nnot a line\n2026-10-22T14:02:00,Fika,udp,ok\n",
        );

        let recent = attempt_log_in(&directory).read_recent(3);

        assert_eq!(
            recent,
            vec![
                record("2026-10-22T14:01:00", "Khalifa", "ikev2", None, false),
                record("2026-10-22T14:02:00", "Fika", "udp", None, true)
            ]
        );
    }

    #[test]
    fn should_read_four_and_five_column_lines() {
        let directory = TemporaryDirectory::new();
        write_log(
            &directory,
            "2026-10-22T14:00:00,Lygos,tcp,ok\n2026-10-22T14:01:00,The 6,tcp,ok,587\n2026-10-22T14:02:00,Fika,udp,fail,\n",
        );

        let recent = attempt_log_in(&directory).read_recent(300);

        assert_eq!(
            recent,
            vec![
                record("2026-10-22T14:00:00", "Lygos", "tcp", None, true),
                record("2026-10-22T14:01:00", "The 6", "tcp", Some(587), true),
                record("2026-10-22T14:02:00", "Fika", "udp", None, false)
            ]
        );
    }

    #[test]
    fn should_skip_a_line_whose_result_is_cut_off() {
        let directory = TemporaryDirectory::new();
        write_log(
            &directory,
            "2026-10-22T14:00:00,Lygos,tcp,o\n2026-10-22T14:01:00,Fika,udp,ok,443\n",
        );

        let recent = attempt_log_in(&directory).read_recent(300);

        assert_eq!(
            recent,
            vec![record(
                "2026-10-22T14:01:00",
                "Fika",
                "udp",
                Some(443),
                true
            )]
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
    fn should_round_trip_last_good_with_a_port() {
        let directory = TemporaryDirectory::new();
        let attempt_log = attempt_log_in(&directory);
        let pair = pair("Stolen Cheese", "wireguard", Some(443));

        attempt_log.write_last_good(&pair);

        assert_eq!(
            fs::read_to_string(paths_in(&directory).last_good_file()).unwrap(),
            "Stolen Cheese|wireguard:443\n"
        );
        assert_eq!(attempt_log.read_last_good(), Some(pair));
    }

    #[test]
    fn should_read_last_good_without_a_port() {
        let directory = TemporaryDirectory::new();
        let paths = paths_in(&directory);
        fs::create_dir_all(&paths.state_directory).unwrap();
        fs::write(paths.last_good_file(), "The 6|tcp\n").unwrap();

        assert_eq!(
            attempt_log_in(&directory).read_last_good(),
            Some(pair("The 6", "tcp", None))
        );
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
            Some("Khalifa|ikev2:port"),
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

mod label {
    use super::*;

    #[test]
    fn should_join_the_protocol_and_port() {
        assert_eq!(pair("The 6", "tcp", Some(443)).label(), "tcp:443");
    }

    #[test]
    fn should_give_the_protocol_alone_without_a_port() {
        assert_eq!(pair("The 6", "tcp", None).label(), "tcp");
    }
}
