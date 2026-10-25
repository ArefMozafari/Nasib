use nasib_linux::output_parser::{parse, parse_status, EngineEvent, OutputTone, TunnelStatus};

fn pair(location: &str, protocol: &str) -> (String, String) {
    (location.to_string(), protocol.to_string())
}

mod parse_line {
    use super::*;

    #[test]
    fn should_parse_trying_connected_and_failed_lines() {
        let (location, protocol) = pair("Stolen Cheese", "ikev2");
        assert_eq!(
            parse("trying: Stolen Cheese / ikev2"),
            EngineEvent::Trying { location, protocol }
        );
        let (location, protocol) = pair("Magic City", "ikev2");
        assert_eq!(
            parse("connected: Magic City / ikev2"),
            EngineEvent::Connected { location, protocol }
        );
        let (location, protocol) = pair("Sausage Party", "wireguard");
        assert_eq!(
            parse("failed: Sausage Party / wireguard"),
            EngineEvent::Failed { location, protocol }
        );
    }

    #[test]
    fn should_parse_the_fixed_lines() {
        assert_eq!(parse("nothing worked"), EngineEvent::NothingWorked);
        assert_eq!(parse("survey finished"), EngineEvent::SurveyFinished);
        assert_eq!(parse("survey stopped"), EngineEvent::SurveyStopped);
        assert_eq!(parse("search stopped"), EngineEvent::SearchStopped);
        assert_eq!(parse("tunnel down"), EngineEvent::TunnelDown);
    }

    #[test]
    fn should_treat_unknown_lines_as_other() {
        let line = "protocol order (from Settings): ikev2, wireguard";
        assert_eq!(parse(line), EngineEvent::Other(line.to_string()));
    }

    #[test]
    fn should_split_the_pair_on_the_last_separator() {
        let (location, protocol) = pair("Inside / Job", "ikev2");
        assert_eq!(
            parse("trying: Inside / Job / ikev2"),
            EngineEvent::Trying { location, protocol }
        );
    }

    #[test]
    fn should_keep_the_port_in_the_protocol() {
        let (location, protocol) = pair("The 6", "tcp:443");
        assert_eq!(
            parse("trying: The 6 / tcp:443"),
            EngineEvent::Trying { location, protocol }
        );
    }

    #[test]
    fn should_parse_cli_missing() {
        assert_eq!(
            parse("windscribe-cli missing: /usr/bin/windscribe-cli"),
            EngineEvent::CliMissing("/usr/bin/windscribe-cli".to_string())
        );
    }

    #[test]
    fn should_parse_phase_lines() {
        assert_eq!(
            parse("phase: tcp on port 443"),
            EngineEvent::Phase("tcp on port 443".to_string())
        );
    }
}

mod tone {
    use super::*;

    #[test]
    fn should_give_each_event_its_tone() {
        let (location, protocol) = pair("Lygos", "tcp");
        assert_eq!(
            EngineEvent::Trying {
                location: location.clone(),
                protocol: protocol.clone()
            }
            .tone(),
            OutputTone::Primary
        );
        assert_eq!(
            EngineEvent::Connected {
                location: location.clone(),
                protocol: protocol.clone()
            }
            .tone(),
            OutputTone::Success
        );
        assert_eq!(
            EngineEvent::Failed { location, protocol }.tone(),
            OutputTone::Muted
        );
        assert_eq!(EngineEvent::TunnelDown.tone(), OutputTone::Warning);
        assert_eq!(EngineEvent::NothingWorked.tone(), OutputTone::Error);
        assert_eq!(
            EngineEvent::CliMissing(String::new()).tone(),
            OutputTone::Error
        );
        assert_eq!(EngineEvent::SearchStopped.tone(), OutputTone::Accent);
        assert_eq!(EngineEvent::SurveyStopped.tone(), OutputTone::Accent);
        assert_eq!(EngineEvent::SurveyFinished.tone(), OutputTone::Accent);
        assert_eq!(EngineEvent::Phase(String::new()).tone(), OutputTone::Phase);
        assert_eq!(EngineEvent::Other(String::new()).tone(), OutputTone::Detail);
    }
}

mod status_line {
    use super::*;

    #[test]
    fn should_parse_a_connected_status_line() {
        assert_eq!(
            parse_status("connected: Atlanta - Magic City / IKEv2"),
            Some(TunnelStatus {
                location: "Atlanta - Magic City".to_string(),
                protocol: "IKEv2".to_string()
            })
        );
    }

    #[test]
    fn should_return_none_for_down_cli_missing_and_empty_output() {
        assert_eq!(parse_status("down"), None);
        assert_eq!(
            parse_status("windscribe-cli missing: /usr/bin/windscribe-cli"),
            None
        );
        assert_eq!(parse_status(""), None);
    }
}
