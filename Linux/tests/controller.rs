use nasib_linux::controller::{
    is_striped, tunnel_dropped, ConnectionState, Controller, Notification,
};

fn connected() -> ConnectionState {
    ConnectionState::Connected {
        location: "Zurich - Altstadt".to_string(),
        protocol: "IKEv2".to_string(),
    }
}

fn searching() -> ConnectionState {
    ConnectionState::Searching {
        location: "Altstadt".to_string(),
        protocol: "ikev2".to_string(),
    }
}

fn notification(title: &str, body: &str) -> Option<Notification> {
    Some(Notification {
        title: title.to_string(),
        body: body.to_string(),
    })
}

mod lines {
    use super::*;

    #[test]
    fn should_show_the_server_being_tried() {
        let mut controller = Controller::new(false);
        controller.handle_line("trying: Lygos / tcp:443");

        assert_eq!(
            controller.state,
            ConnectionState::Searching {
                location: "Lygos".to_string(),
                protocol: "tcp:443".to_string()
            }
        );
    }

    #[test]
    fn should_connect_and_notify() {
        let mut controller = Controller::new(false);

        let posted = controller.handle_line("connected: Lygos / tcp:443");

        assert_eq!(
            controller.state,
            ConnectionState::Connected {
                location: "Lygos".to_string(),
                protocol: "tcp:443".to_string()
            }
        );
        assert_eq!(posted, notification("Connected", "Lygos / tcp:443"));
    }

    #[test]
    fn should_go_down_and_notify_when_nothing_worked() {
        let mut controller = Controller::new(false);

        let posted = controller.handle_line("nothing worked");

        assert_eq!(controller.state, ConnectionState::Down);
        assert_eq!(
            posted,
            notification("No working server", "Every datacenter and protocol failed.")
        );
    }

    #[test]
    fn should_start_finding_and_notify_when_the_tunnel_goes_down() {
        let mut controller = Controller::new(false);

        let posted = controller.handle_line("tunnel down");

        assert_eq!(controller.state, ConnectionState::Down);
        assert!(controller.is_finding);
        assert_eq!(
            posted,
            notification("Tunnel down", "Searching for a working server…")
        );
    }

    #[test]
    fn should_mark_the_cli_missing() {
        let mut controller = Controller::new(false);

        let posted = controller.handle_line("windscribe-cli missing: /usr/bin/windscribe-cli");

        assert_eq!(controller.state, ConnectionState::Down);
        assert!(controller.cli_missing);
        assert_eq!(posted, None);
    }

    #[test]
    fn should_keep_the_state_for_other_lines_without_notifying() {
        for line in [
            "failed: Lygos / tcp:443",
            "search stopped",
            "survey stopped",
            "survey finished",
            "phase: tcp on port 443",
            "protocol order (from Settings): tcp",
        ] {
            let mut controller = Controller::new(false);
            controller.handle_line("trying: Lygos / tcp:443");

            let posted = controller.handle_line(line);

            assert_eq!(
                controller.state,
                ConnectionState::Searching {
                    location: "Lygos".to_string(),
                    protocol: "tcp:443".to_string()
                },
                "{line}"
            );
            assert_eq!(posted, None, "{line}");
        }
    }
}

mod recent_lines {
    use super::*;

    #[test]
    fn should_keep_the_last_two_hundred_lines() {
        let mut controller = Controller::new(false);
        for index in 0..205 {
            controller.handle_line(&format!("line {index}"));
        }

        assert_eq!(controller.recent_lines.len(), 200);
        assert_eq!(
            controller.recent_lines.front().map(String::as_str),
            Some("line 5")
        );
        assert_eq!(controller.dropped_line_count, 5);
    }

    #[test]
    fn should_count_the_lines_not_shown_yet() {
        let mut controller = Controller::new(false);
        controller.handle_line("one");
        controller.handle_line("two");
        assert_eq!(controller.new_line_count(0), 2);
        assert_eq!(controller.new_line_count(2), 0);

        controller.handle_line("three");

        assert_eq!(controller.new_line_count(2), 1);
    }

    // When more lines arrive than the pane keeps, only the kept ones are new.
    #[test]
    fn should_cap_the_new_lines_at_the_kept_ones() {
        let mut controller = Controller::new(false);
        for index in 0..250 {
            controller.handle_line(&format!("line {index}"));
        }

        assert_eq!(controller.new_line_count(10), 200);
        assert_eq!(controller.shown_line_total(), 250);
    }

    #[test]
    fn should_stripe_every_other_line() {
        assert!(!is_striped(0, 0));
        assert!(is_striped(1, 0));
    }

    // Once the oldest line is dropped, each remaining line keeps its stripe instead of every row flipping.
    #[test]
    fn should_keep_each_lines_stripe_when_older_lines_are_dropped() {
        assert_eq!(is_striped(1, 0), is_striped(0, 1));
    }
}

mod runs {
    use super::*;

    #[test]
    fn should_mark_a_find_until_the_engine_exits() {
        let mut controller = Controller::new(false);
        controller.begin_find();
        assert!(controller.is_busy && controller.is_finding);

        controller.engine_exited();

        assert!(!controller.is_busy && !controller.is_finding && !controller.is_surveying);
    }

    #[test]
    fn should_mark_a_survey_until_the_engine_exits() {
        let mut controller = Controller::new(false);
        controller.begin_survey();
        assert!(controller.is_busy && controller.is_surveying);

        controller.engine_exited();

        assert!(!controller.is_surveying);
    }

    // A periodic watch is busy for a moment but isn't a search, so the Find button doesn't flicker.
    #[test]
    fn should_not_mark_a_watch_as_finding() {
        let mut controller = Controller::new(false);
        controller.begin_watch();

        assert!(controller.is_busy && !controller.is_finding);
    }

    #[test]
    fn should_not_run_without_the_engine() {
        let controller = Controller::new(true);

        assert!(controller.engine_missing);
        assert!(!controller.can_run());
    }

    // Without the engine no status read ever lands, so the window says down instead of checking forever.
    #[test]
    fn should_start_down_without_the_engine() {
        assert_eq!(Controller::new(true).state, ConnectionState::Down);
        assert_eq!(Controller::new(false).state, ConnectionState::Unknown);
    }

    #[test]
    fn should_not_run_while_busy() {
        let mut controller = Controller::new(false);
        assert!(controller.can_run());

        controller.begin_find();

        assert!(!controller.can_run());
    }
}

mod status {
    use super::*;

    #[test]
    fn should_show_the_connected_server() {
        let mut controller = Controller::new(false);

        controller.apply_status("connected: Zurich - Altstadt / IKEv2");

        assert_eq!(controller.state, connected());
    }

    #[test]
    fn should_report_a_drop_from_connected_to_down() {
        let mut controller = Controller::new(false);
        controller.apply_status("connected: Zurich - Altstadt / IKEv2");

        assert!(controller.apply_status("down").dropped);
        assert_eq!(controller.state, ConnectionState::Down);
    }

    // The window redraws only on a real change, so the output pane keeps its scroll and selection.
    #[test]
    fn should_report_no_change_for_the_same_status() {
        let mut controller = Controller::new(false);
        assert!(controller.apply_status("down").changed);

        assert!(!controller.apply_status("down").changed);
        assert!(
            controller
                .apply_status("connected: Zurich - Altstadt / IKEv2")
                .changed
        );
    }

    #[test]
    fn should_report_a_change_when_the_cli_goes_missing() {
        let mut controller = Controller::new(false);
        controller.apply_status("down");

        assert!(
            controller
                .apply_status("windscribe-cli missing: /usr/bin/windscribe-cli")
                .changed
        );
    }

    #[test]
    fn should_mark_and_clear_the_missing_cli() {
        let mut controller = Controller::new(false);

        controller.apply_status("windscribe-cli missing: /usr/bin/windscribe-cli");
        assert!(controller.cli_missing);

        controller.apply_status("down");
        assert!(!controller.cli_missing);
    }

    // A run's own lines are newer than a status read that started before it.
    #[test]
    fn should_ignore_status_while_busy() {
        let mut controller = Controller::new(false);
        controller.begin_find();
        controller.handle_line("trying: Lygos / tcp:443");

        let update = controller.apply_status("down");
        assert!(!update.dropped && !update.changed);
        assert!(matches!(
            controller.state,
            ConnectionState::Searching { .. }
        ));
    }
}

mod drops {
    use super::*;

    #[test]
    fn should_detect_drop_when_connected_tunnel_goes_down() {
        assert!(tunnel_dropped(&connected(), &ConnectionState::Down));
    }

    #[test]
    fn should_not_detect_drop_while_still_connected() {
        assert!(!tunnel_dropped(&connected(), &connected()));
    }

    #[test]
    fn should_not_detect_drop_when_tunnel_was_never_connected() {
        assert!(!tunnel_dropped(
            &ConnectionState::Unknown,
            &ConnectionState::Down
        ));
        assert!(!tunnel_dropped(
            &ConnectionState::Down,
            &ConnectionState::Down
        ));
    }

    #[test]
    fn should_not_detect_drop_after_stopped_search() {
        assert!(!tunnel_dropped(&searching(), &ConnectionState::Down));
    }
}
