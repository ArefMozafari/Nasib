use std::path::PathBuf;

use nasib_engine::attempt_log::ServerPair;
use nasib_engine::events::EngineEvent;

fn pair(nickname: &str, protocol: &str) -> ServerPair {
    ServerPair {
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        port: None,
    }
}

mod text {
    use super::*;

    #[test]
    fn should_print_the_port_after_the_protocol() {
        let pair = ServerPair {
            port: Some(443),
            ..pair("The 6", "tcp")
        };

        assert_eq!(
            EngineEvent::Trying(pair.clone()).text(),
            "trying: The 6 / tcp:443"
        );
        assert_eq!(
            EngineEvent::Connected(pair.clone()).text(),
            "connected: The 6 / tcp:443"
        );
        assert_eq!(EngineEvent::Failed(pair).text(), "failed: The 6 / tcp:443");
    }

    #[test]
    fn should_print_the_output_contract_lines() {
        let cases = [
            (
                EngineEvent::Trying(pair("Stolen Cheese", "ikev2")),
                "trying: Stolen Cheese / ikev2",
            ),
            (
                EngineEvent::Connected(pair("Khalifa", "wireguard")),
                "connected: Khalifa / wireguard",
            ),
            (
                EngineEvent::Failed(pair("Lygos", "tcp")),
                "failed: Lygos / tcp",
            ),
            (EngineEvent::SearchStopped, "search stopped"),
            (EngineEvent::SurveyStopped, "survey stopped"),
            (EngineEvent::SurveyFinished, "survey finished"),
            (EngineEvent::NothingWorked, "nothing worked"),
            (EngineEvent::TunnelDown, "tunnel down"),
            (EngineEvent::SearchAlreadyRunning, "search already running"),
            (
                EngineEvent::CliMissing(PathBuf::from("/usr/local/bin/windscribe-cli")),
                "windscribe-cli missing: /usr/local/bin/windscribe-cli",
            ),
            (
                EngineEvent::Message("protocol order (from Settings): ikev2".to_string()),
                "protocol order (from Settings): ikev2",
            ),
        ];
        for (event, expected) in cases {
            assert_eq!(event.text(), expected);
        }
    }
}
