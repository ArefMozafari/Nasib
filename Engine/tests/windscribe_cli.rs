mod support;

use nasib_engine::windscribe_cli::{self, ProcessWindscribeCli, WindscribeCli};
use support::TemporaryDirectory;

mod arguments {
    use super::*;

    #[test]
    fn should_connect_without_a_port_when_none_is_given() {
        assert_eq!(
            windscribe_cli::connect_arguments("Stolen Cheese", "ikev2", None),
            ["connect", "-n", "Stolen Cheese", "ikev2"]
        );
    }

    #[test]
    fn should_connect_on_the_given_port() {
        assert_eq!(
            windscribe_cli::connect_arguments("The 6", "tcp", Some(587)),
            ["connect", "-n", "The 6", "tcp:587"]
        );
    }

    #[test]
    fn should_ask_for_a_protocols_ports() {
        assert_eq!(windscribe_cli::ports_arguments("tcp"), ["ports", "tcp"]);
    }

    #[test]
    fn should_ask_for_the_status() {
        assert_eq!(windscribe_cli::STATUS_ARGUMENTS, ["status"]);
    }

    #[test]
    fn should_disconnect_and_wait_for_it() {
        assert_eq!(windscribe_cli::DISCONNECT_ARGUMENTS, ["disconnect"]);
    }
}

mod parse_ports {
    use super::*;

    #[test]
    fn should_parse_the_ports_after_the_log_lines() {
        let output = "{\"tm\": \"2026-10-25 13:02:18.531\", \"lvl\": \"info\", \"msg\": \"=== Started ===\"}\n443, 587, 21, 22\n";

        assert_eq!(windscribe_cli::parse_ports(output), vec![443, 587, 21, 22]);
    }

    #[test]
    fn should_parse_a_single_port() {
        assert_eq!(windscribe_cli::parse_ports("500\n"), vec![500]);
    }

    #[test]
    fn should_return_no_ports_for_unexpected_output() {
        assert!(windscribe_cli::parse_ports("Unknown protocol\n").is_empty());
        assert!(windscribe_cli::parse_ports("").is_empty());
        assert!(windscribe_cli::parse_ports("443, port, 22\n").is_empty());
    }

    #[test]
    fn should_return_no_ports_when_the_cli_is_busy() {
        assert!(windscribe_cli::parse_ports("Windscribe CLI is already running\n").is_empty());
    }
}

mod missing_cli {
    use super::*;

    #[test]
    fn should_return_nothing_when_the_cli_is_not_installed() {
        let directory = TemporaryDirectory::new();
        let cli = ProcessWindscribeCli::new(directory.path().join("windscribe-cli"));

        assert!(!cli.is_installed());
        assert_eq!(cli.read_status(), "");
    }
}

#[cfg(unix)]
mod running {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn should_return_standard_output_and_error_together() {
        let directory = TemporaryDirectory::new();
        let script = directory.path().join("windscribe-cli");
        fs::write(&script, "#!/bin/sh\necho \"out $*\"\necho err >&2\n").unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        let cli = ProcessWindscribeCli::new(script);

        let output = cli.read_status();

        assert!(cli.is_installed());
        assert!(output.contains("out status"), "{output}");
        assert!(output.contains("err"), "{output}");
    }
}
