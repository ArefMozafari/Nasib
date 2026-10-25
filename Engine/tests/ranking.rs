use nasib_engine::attempt_log::AttemptRecord;
use nasib_engine::ranking::{self, PortOrder};

fn attempt(nickname: &str, protocol: &str, succeeded: bool) -> AttemptRecord {
    AttemptRecord {
        timestamp: "2026-10-22T14:00:00".to_string(),
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        port: None,
        succeeded,
    }
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

mod rank {
    use super::*;

    #[test]
    fn should_put_the_best_success_rate_first() {
        let recent = vec![
            attempt("Lygos", "tcp", false),
            attempt("Lygos", "udp", false),
            attempt("Khalifa", "tcp", true),
            attempt("Khalifa", "udp", false),
            attempt("Fika", "tcp", true),
        ];

        let ranked = ranking::rank(&names(&["Lygos", "Khalifa", "Fika"]), &recent);

        assert_eq!(ranked, names(&["Fika", "Khalifa", "Lygos"]));
    }

    #[test]
    fn should_keep_the_given_order_for_ties_and_names_never_tried() {
        let recent = vec![attempt("Fika", "tcp", false)];

        let ranked = ranking::rank(&names(&["Khalifa", "Lygos", "Vitoshka", "Fika"]), &recent);

        assert_eq!(ranked, names(&["Khalifa", "Lygos", "Vitoshka", "Fika"]));
    }

    #[test]
    fn should_count_every_protocol_tried_on_a_datacenter_towards_its_rate() {
        let recent = vec![
            attempt("Khalifa", "wireguard", true),
            attempt("Khalifa", "ikev2", false),
            attempt("Khalifa", "tcp", false),
            attempt("Lygos", "wireguard", true),
            attempt("Lygos", "ikev2", true),
        ];

        let ranked = ranking::rank(&names(&["Khalifa", "Lygos"]), &recent);

        assert_eq!(ranked, names(&["Lygos", "Khalifa"]));
    }

    #[test]
    fn should_treat_rates_equal_to_three_decimal_places_as_ties() {
        let mut recent = vec![attempt("Khalifa", "tcp", true)];
        recent.extend((0..1999).map(|_| attempt("Khalifa", "tcp", true)));
        recent.push(attempt("Khalifa", "tcp", false));
        recent.push(attempt("Lygos", "tcp", true));

        let ranked = ranking::rank(&names(&["Khalifa", "Lygos"]), &recent);

        assert_eq!(ranked, names(&["Khalifa", "Lygos"]));
    }
}

fn on_port(protocol: &str, port: Option<u16>, succeeded: bool) -> AttemptRecord {
    AttemptRecord {
        port,
        ..attempt("The 6", protocol, succeeded)
    }
}

/// Windscribe's tcp ports, default first.
const TCP_PORTS: [u16; 5] = [443, 587, 21, 22, 80];

mod rank_ports {
    use super::*;

    #[test]
    fn should_prefer_the_default_port_without_history() {
        let order = ranking::rank_ports("tcp", &TCP_PORTS, &[]);

        assert_eq!(
            order,
            PortOrder {
                preferred: Some(443),
                others: vec![587, 21, 22, 80]
            }
        );
    }

    #[test]
    fn should_prefer_a_port_that_worked_over_the_default() {
        let recent = vec![
            on_port("tcp", Some(443), false),
            on_port("tcp", Some(587), true),
        ];

        let order = ranking::rank_ports("tcp", &TCP_PORTS, &recent);

        assert_eq!(order.preferred, Some(587));
        assert_eq!(order.others, vec![21, 22, 80, 443]);
    }

    #[test]
    fn should_count_records_without_a_port_as_the_default() {
        let recent = vec![on_port("tcp", None, true), on_port("tcp", Some(587), false)];

        let order = ranking::rank_ports("tcp", &TCP_PORTS, &recent);

        assert_eq!(order.preferred, Some(443));
    }

    #[test]
    fn should_put_untried_ports_before_ports_that_only_failed() {
        let recent = vec![on_port("tcp", Some(21), false)];

        let order = ranking::rank_ports("tcp", &TCP_PORTS, &recent);

        assert_eq!(order.others, vec![587, 22, 80, 21]);
    }

    #[test]
    fn should_put_ports_that_worked_before_untried_ones() {
        let recent = vec![
            on_port("tcp", Some(443), true),
            on_port("tcp", Some(80), true),
            on_port("tcp", Some(80), false),
        ];

        let order = ranking::rank_ports("tcp", &TCP_PORTS, &recent);

        assert_eq!(order.preferred, Some(443));
        assert_eq!(order.others, vec![80, 587, 21, 22]);
    }

    #[test]
    fn should_put_less_tried_failing_ports_first() {
        let recent = vec![
            on_port("tcp", Some(21), false),
            on_port("tcp", Some(21), false),
            on_port("tcp", Some(22), false),
        ];

        let order = ranking::rank_ports("tcp", &TCP_PORTS, &recent);

        assert_eq!(order.others, vec![587, 80, 22, 21]);
    }

    #[test]
    fn should_return_no_port_when_the_cli_lists_none() {
        assert_eq!(
            ranking::rank_ports("tcp", &[], &[]),
            PortOrder {
                preferred: None,
                others: vec![]
            }
        );
    }

    #[test]
    fn should_ignore_other_protocols() {
        let recent = vec![on_port("udp", Some(587), true)];

        assert_eq!(
            ranking::rank_ports("tcp", &TCP_PORTS, &recent).preferred,
            Some(443)
        );
    }
}
