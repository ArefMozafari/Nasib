mod support;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use nasib_engine::stop::StopSignal;
use nasib_engine::timings::Timings;
use nasib_engine::tunnel::Tunnel;
use nasib_engine::tunnel_status::ConnectState;
use support::{FakeBlockedSiteProbe, FakeCli, FakeTunnelState, DISCONNECT_CALL};

/// A thousandth of the real timings, so every poll count matches production without real waiting.
const TIME_SCALE: f64 = 1000.0;
const SETTLE_TIME_SCALE: f64 = 10.0;
const CONNECT_POLLS: usize = 10;
const DISCONNECT_POLLS: usize = 15;

fn create() -> (Tunnel, Arc<FakeCli>, Arc<FakeBlockedSiteProbe>) {
    let cli = FakeCli::new();
    let probe = FakeBlockedSiteProbe::new(cli.clone());
    (
        Tunnel::new(
            cli.clone(),
            probe.clone(),
            Timings::default().scaled(TIME_SCALE),
        ),
        cli,
        probe,
    )
}

mod disconnect_and_wait {
    use super::*;

    #[test]
    fn should_return_at_once_when_the_tunnel_goes_down() {
        let (tunnel, cli, _) = create();
        cli.with(|state| state.state = FakeTunnelState::Connected);

        assert_eq!(tunnel.disconnect_and_wait(&StopSignal::new()), Ok(true));
        assert_eq!(cli.status_reads_after_last(DISCONNECT_CALL), 1);
    }

    #[test]
    fn should_end_disconnect_wait_on_error() {
        let (tunnel, cli, _) = create();
        cli.with(|state| {
            state.state = FakeTunnelState::Error;
            state.error_is_sticky = true;
        });

        assert_eq!(tunnel.disconnect_and_wait(&StopSignal::new()), Ok(true));
        assert_eq!(cli.status_reads_after_last(DISCONNECT_CALL), 1);
    }

    #[test]
    fn should_give_up_disconnect_after_fifteen_polls() {
        let (tunnel, cli, _) = create();
        cli.with(|state| state.stuck_connected = true);

        assert_eq!(tunnel.disconnect_and_wait(&StopSignal::new()), Ok(false));
        assert_eq!(
            cli.status_reads_after_last(DISCONNECT_CALL),
            DISCONNECT_POLLS
        );
    }
}

mod connect {
    use super::*;

    #[test]
    fn should_disconnect_first_when_a_tunnel_is_up() {
        let (tunnel, cli, _) = create();
        cli.with(|state| state.state = FakeTunnelState::Connected);

        tunnel
            .connect("Khalifa", "ikev2", &StopSignal::new())
            .unwrap();

        assert_eq!(cli.count_calls(DISCONNECT_CALL), 1);
        assert_eq!(cli.state(), FakeTunnelState::Connected);
    }

    #[test]
    fn should_not_disconnect_when_the_cli_reports_an_error() {
        let (tunnel, cli, _) = create();
        cli.with(|state| state.state = FakeTunnelState::Error);

        tunnel
            .connect("Khalifa", "ikev2", &StopSignal::new())
            .unwrap();

        assert_eq!(cli.count_calls(DISCONNECT_CALL), 0);
    }

    #[test]
    fn should_stop_polling_once_connected() {
        let (tunnel, cli, _) = create();

        tunnel
            .connect("Khalifa", "ikev2", &StopSignal::new())
            .unwrap();

        assert_eq!(cli.status_reads_after_last("connect "), 1);
    }

    #[test]
    fn should_stop_polling_connect_after_ten_polls() {
        let (tunnel, cli, _) = create();
        cli.with(|state| state.pair_connects = Box::new(|_, _| false));

        tunnel
            .connect("Khalifa", "ikev2", &StopSignal::new())
            .unwrap();

        assert_eq!(cli.status_reads_after_last("connect "), CONNECT_POLLS);
    }

    #[test]
    fn should_settle_after_connecting() {
        // A tenth of the real timings keeps the 2-second settle long enough to measure.
        let timings = Timings::default().scaled(SETTLE_TIME_SCALE);
        let cli = FakeCli::new();
        let tunnel = Tunnel::new(cli.clone(), FakeBlockedSiteProbe::new(cli.clone()), timings);
        let started = std::time::Instant::now();

        tunnel
            .connect("Khalifa", "ikev2", &StopSignal::new())
            .unwrap();

        assert!(started.elapsed() >= timings.status_poll_interval + timings.settle_delay);
    }

    #[test]
    fn should_pass_the_nickname_and_protocol_to_the_cli() {
        let (tunnel, cli, _) = create();

        tunnel
            .connect("Stolen Cheese", "wstunnel", &StopSignal::new())
            .unwrap();

        assert_eq!(cli.connect_calls(), ["connect Stolen Cheese wstunnel"]);
    }

    #[test]
    fn should_return_stopped_without_connecting_after_a_stop() {
        let (tunnel, cli, _) = create();
        let stop = StopSignal::new();
        stop.request();

        assert!(tunnel.connect("Khalifa", "ikev2", &stop).is_err());
        assert!(cli.connect_calls().len() <= 1);
    }
}

mod works {
    use super::*;

    #[test]
    fn should_not_probe_when_the_tunnel_is_down() {
        let (tunnel, _, probe) = create();

        assert_eq!(tunnel.works(&StopSignal::new()), Ok(false));
        assert_eq!(probe.probe_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn should_work_only_when_the_blocked_site_loads() {
        let (tunnel, cli, probe) = create();
        cli.with(|state| state.state = FakeTunnelState::Connected);
        probe.set_loads_through(|_| false);

        assert_eq!(tunnel.works(&StopSignal::new()), Ok(false));

        probe.set_loads_through(|_| true);

        assert_eq!(tunnel.works(&StopSignal::new()), Ok(true));
    }
}

mod read_status {
    use super::*;

    #[test]
    fn should_parse_what_the_cli_prints() {
        let (tunnel, cli, _) = create();
        cli.with(|state| {
            state.state = FakeTunnelState::Connected;
            state.connected_location = "Amsterdam - Stolen Cheese".to_string();
        });

        let status = tunnel.read_status();

        assert_eq!(status.state, ConnectState::Connected);
        assert_eq!(
            status.location.as_deref(),
            Some("Amsterdam - Stolen Cheese")
        );
        assert_eq!(status.protocol.as_deref(), Some("IKEv2"));
    }
}
