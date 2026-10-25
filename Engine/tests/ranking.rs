use nasib_engine::attempt_log::AttemptRecord;
use nasib_engine::ranking;

fn attempt(nickname: &str, protocol: &str, succeeded: bool) -> AttemptRecord {
    AttemptRecord {
        timestamp: "2026-10-22T14:00:00".to_string(),
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
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
