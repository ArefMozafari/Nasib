use nasib_engine::tunnel_status::{ConnectState, TunnelStatus};

const CONNECTED_OUTPUT: &str = "Internet connectivity: available\nLogin state: Logged in\nConnect state: Connected: Atlanta - Magic City\nProtocol: IKEv2:500\nVPN IP: 198.51.100.20\n";

mod parse {
    use super::*;

    #[test]
    fn should_read_the_port_from_the_status() {
        assert_eq!(TunnelStatus::parse(CONNECTED_OUTPUT).port, Some(500));
    }

    #[test]
    fn should_read_no_port_without_a_protocol_line() {
        assert_eq!(
            TunnelStatus::parse("Connect state: Disconnected\n").port,
            None
        );
    }

    #[test]
    fn should_parse_a_connected_status_with_the_protocol_port_stripped() {
        let status = TunnelStatus::parse(CONNECTED_OUTPUT);

        assert_eq!(status.state, ConnectState::Connected);
        assert_eq!(status.location.as_deref(), Some("Atlanta - Magic City"));
        assert_eq!(status.protocol.as_deref(), Some("IKEv2"));
        assert!(status.is_connected());
        assert!(!status.is_down());
    }

    #[test]
    fn should_parse_a_disconnected_status_with_windows_line_endings() {
        let status = TunnelStatus::parse(
            "Firewall state: Off\r\nConnect state: Disconnected\r\nPublic IP: 203.0.113.10\r\n",
        );

        assert_eq!(status.state, ConnectState::Disconnected);
        assert!(status.is_down());
        assert!(!status.is_connected());
    }

    #[test]
    fn should_count_an_error_as_down_and_not_connected() {
        let status = TunnelStatus::parse("Connect state: Error: 11\n");

        assert_eq!(status.state, ConnectState::Error);
        assert_eq!(status.location, None);
        assert!(status.is_down());
        assert!(!status.is_connected());
    }

    #[test]
    fn should_treat_transitional_states_as_neither_up_nor_down() {
        for (output, expected) in [
            ("Connect state: Connecting", ConnectState::Connecting),
            ("Connect state: Disconnecting", ConnectState::Disconnecting),
        ] {
            let status = TunnelStatus::parse(output);

            assert_eq!(status.state, expected);
            assert!(!status.is_connected());
            assert!(!status.is_down());
        }
    }

    #[test]
    fn should_report_unknown_for_empty_output() {
        let status = TunnelStatus::parse("");

        assert_eq!(status.state, ConnectState::Unknown);
        assert!(!status.is_down());
        assert!(!status.is_connected());
    }
}

mod status_line {
    use super::*;

    #[test]
    fn should_print_the_location_and_protocol_when_connected() {
        assert_eq!(
            TunnelStatus::parse(CONNECTED_OUTPUT).status_line(),
            "connected: Atlanta - Magic City / IKEv2"
        );
    }

    #[test]
    fn should_print_down_when_disconnected() {
        assert_eq!(
            TunnelStatus::parse("Connect state: Disconnected\nPublic IP: 203.0.113.10")
                .status_line(),
            "down"
        );
    }

    #[test]
    fn should_print_down_while_connecting() {
        assert_eq!(
            TunnelStatus::parse("Connect state: Connecting\nProtocol: IKEv2:500").status_line(),
            "down"
        );
    }

    #[test]
    fn should_print_down_when_connected_without_a_protocol() {
        assert_eq!(
            TunnelStatus::parse("Connect state: Connected: Atlanta - Magic City\n").status_line(),
            "down"
        );
    }
}
