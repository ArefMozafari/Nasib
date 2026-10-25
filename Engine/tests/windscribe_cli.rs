mod support;

use nasib_engine::windscribe_cli::{self, ProcessWindscribeCli, WindscribeCli};
use support::TemporaryDirectory;

mod arguments {
    use super::*;

    #[test]
    fn should_connect_without_blocking_by_nickname_and_protocol() {
        assert_eq!(
            windscribe_cli::connect_arguments("Stolen Cheese", "ikev2"),
            ["connect", "-n", "Stolen Cheese", "ikev2"]
        );
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
