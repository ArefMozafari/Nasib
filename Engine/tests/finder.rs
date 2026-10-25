mod support;

use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nasib_engine::attempt_log::{AttemptLog, Clock, ServerPair};
use nasib_engine::configuration::{ALL_PROTOCOLS, DEFAULT_LOCATIONS};
use nasib_engine::datacenter::Datacenter;
use nasib_engine::events::EngineEvent;
use nasib_engine::finder::{Finder, Outcome};
use nasib_engine::paths::Paths;
use nasib_engine::reachability::ReachabilityProbe;
use nasib_engine::stop::StopSignal;
use nasib_engine::timings::Timings;
use nasib_engine::tunnel::Tunnel;
use support::{
    FakeBlockedSiteProbe, FakeCli, FakeReachabilityProbe, FakeTunnelState, TemporaryDirectory,
    DISCONNECT_CALL,
};

const CITY: &str = "Dubai";
const NICKNAME: &str = "Khalifa";
const PROTOCOL: &str = "ikev2";
const ADDRESS: &str = "135.136.7.178";
const TIME_SCALE: f64 = 1000.0;
const PROMPT_LIMIT: Duration = Duration::from_secs(2);

struct FixedClock;

impl Clock for FixedClock {
    fn local_timestamp(&self) -> String {
        "2026-10-22T14:40:24".to_string()
    }
}

fn pair(nickname: &str, protocol: &str) -> ServerPair {
    ServerPair {
        nickname: nickname.to_string(),
        protocol: protocol.to_string(),
        port: None,
    }
}

/// A finder wired to fakes in a throwaway state folder, with Khalifa in Dubai as the only bundled datacenter.
struct Harness {
    /// Kept so the state folder lives as long as the harness.
    _directory: TemporaryDirectory,
    paths: Paths,
    cli: Arc<FakeCli>,
    probe: Arc<FakeBlockedSiteProbe>,
    events: Arc<Mutex<Vec<EngineEvent>>>,
    finder: Arc<Finder>,
}

impl Harness {
    fn new() -> Harness {
        Harness::with(vec![], FakeReachabilityProbe::answering_all())
    }

    fn with(extra_seed: Vec<Datacenter>, reachability: Arc<dyn ReachabilityProbe>) -> Harness {
        let mut bundled = extra_seed;
        bundled.push(
            Datacenter::from_location_name(
                &format!("{CITY} - {NICKNAME}"),
                vec![ADDRESS.to_string()],
            )
            .unwrap(),
        );
        Harness::with_bundled(bundled, reachability)
    }

    fn with_bundled(bundled: Vec<Datacenter>, reachability: Arc<dyn ReachabilityProbe>) -> Harness {
        let directory = TemporaryDirectory::new();
        let state = directory.path().join("state");
        let logs = directory.path().join("logs");
        let paths = Paths::from_variables(|name| match name {
            "NASIB_STATE_DIR" => Some(state.to_string_lossy().into_owned()),
            "NASIB_WINDSCRIBE_LOG_DIR" => Some(logs.to_string_lossy().into_owned()),
            _ => None,
        });
        let cli = FakeCli::new();
        let probe = FakeBlockedSiteProbe::new(cli.clone());
        let tunnel = Tunnel::new(
            cli.clone(),
            probe.clone(),
            Timings::default().scaled(TIME_SCALE),
        );
        let events = Arc::new(Mutex::new(Vec::new()));
        let recorded = events.clone();
        let finder = Arc::new(Finder::new(
            paths.clone(),
            bundled,
            tunnel,
            reachability,
            AttemptLog::new(paths.clone(), Box::new(FixedClock)),
            Box::new(move |event| recorded.lock().unwrap().push(event)),
        ));
        Harness {
            _directory: directory,
            paths,
            cli,
            probe,
            events,
            finder,
        }
    }

    fn attempt_log(&self) -> AttemptLog {
        AttemptLog::new(self.paths.clone(), Box::new(FixedClock))
    }

    fn save_configuration(&self, cities: &[&str], protocols: &[&str]) {
        fs::create_dir_all(&self.paths.state_directory).unwrap();
        let text = serde_json::json!({ "cities": cities, "protocolOrder": protocols }).to_string();
        fs::write(self.paths.configuration_file(), text).unwrap();
    }

    fn save_locations(&self, locations: &[&str], protocols: &[&str], rank_by_success: bool) {
        fs::create_dir_all(&self.paths.state_directory).unwrap();
        let text = serde_json::json!({
            "locations": locations,
            "rankBySuccess": rank_by_success,
            "protocolOrder": protocols,
        })
        .to_string();
        fs::write(self.paths.configuration_file(), text).unwrap();
    }

    fn write_last_good(&self, nickname: &str, protocol: &str) {
        self.attempt_log()
            .write_last_good(&pair(nickname, protocol));
    }

    fn record_attempt(&self, nickname: &str, protocol: &str, succeeded: bool) {
        self.attempt_log()
            .record(&pair(nickname, protocol), succeeded);
    }

    /// The log lines as `nickname,protocol,result`, without their timestamps and ports.
    fn logged_attempts(&self) -> Vec<String> {
        fs::read_to_string(self.paths.log_file())
            .unwrap_or_default()
            .lines()
            .map(|line| {
                line.split(',')
                    .skip(1)
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect()
    }

    fn set_ports(&self, protocol: &str, ports: &str) {
        let (protocol, ports) = (protocol.to_string(), format!("{ports}\n"));
        self.cli.with(move |state| {
            state.ports_by_protocol.insert(protocol, ports);
        });
    }

    fn write_log(&self, contents: &str) {
        fs::create_dir_all(&self.paths.state_directory).unwrap();
        fs::write(self.paths.log_file(), contents).unwrap();
    }

    fn phases(&self) -> Vec<String> {
        self.event_texts()
            .into_iter()
            .filter(|text| text.starts_with("phase: "))
            .collect()
    }

    fn event_texts(&self) -> Vec<String> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .map(EngineEvent::text)
            .collect()
    }

    fn last_event(&self) -> EngineEvent {
        self.events
            .lock()
            .unwrap()
            .last()
            .cloned()
            .expect("an event was raised")
    }
}

fn sofia() -> Datacenter {
    Datacenter::from_location_name("Sofia - Vitoshka", vec!["150.241.139.14".to_string()]).unwrap()
}

mod find {
    use super::*;

    // Regression for issue #2: Find Now while connected must not burn an attempt on the last-good pair.
    #[test]
    fn should_log_one_attempt_when_searching_while_connected() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.state = FakeTunnelState::Connected);
        harness.write_last_good(NICKNAME, PROTOCOL);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Succeeded);
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,ok"]);
    }

    #[test]
    fn should_log_one_attempt_when_searching_while_disconnected() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Succeeded);
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,ok"]);
    }

    #[test]
    fn should_not_add_a_disconnect_wait_when_the_cli_keeps_reporting_an_error() {
        let harness = Harness::new();
        harness.cli.with(|state| {
            state.state = FakeTunnelState::Error;
            state.error_is_sticky = true;
            state.pair_connects = Box::new(|_, _| false);
        });
        harness.save_configuration(&[CITY], &[PROTOCOL]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.status_reads_after_last(DISCONNECT_CALL), 1);
    }

    #[test]
    fn should_stop_at_the_first_pair_that_works() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, protocol| protocol == "tcp"));
        harness.save_configuration(&[CITY], &["ikev2", "tcp", "udp"]);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Succeeded);
        assert_eq!(
            harness.logged_attempts(),
            ["Khalifa,ikev2,fail", "Khalifa,tcp,ok"]
        );
        // No ports are listed, so the pair keeps the port the status reports, which the fake gives as 500.
        assert_eq!(
            harness.attempt_log().read_last_good(),
            Some(ServerPair {
                port: Some(500),
                ..pair(NICKNAME, "tcp")
            })
        );
    }

    #[test]
    fn should_count_a_connect_only_when_the_blocked_site_loads() {
        let harness = Harness::new();
        harness.probe.set_loads_through(|_| false);
        harness.save_configuration(&[CITY], &[PROTOCOL]);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Failed);
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,fail"]);
        assert_eq!(harness.last_event(), EngineEvent::NothingWorked);
    }

    #[test]
    fn should_try_every_datacenter_when_no_address_answers() {
        let harness = Harness::with(vec![sofia()], FakeReachabilityProbe::answering(&[]));
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, _| false));
        harness.save_configuration(&[], &[PROTOCOL]);

        harness.finder.find(&StopSignal::new());

        assert!(harness
            .event_texts()
            .contains(&"no address answered; trying all locations".to_string()));
        // A file from before locations existed searches the default locations first, in their order.
        assert_eq!(
            harness.logged_attempts(),
            ["Khalifa,ikev2,fail", "Vitoshka,ikev2,fail"]
        );
    }

    #[test]
    fn should_skip_datacenters_where_no_address_answers() {
        let harness = Harness::with(vec![sofia()], FakeReachabilityProbe::answering(&[ADDRESS]));
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, _| false));
        harness.save_configuration(&[], &[PROTOCOL]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,fail"]);
    }

    #[test]
    fn should_try_datacenters_with_the_best_recent_success_rate_first() {
        let harness = Harness::with(vec![sofia()], FakeReachabilityProbe::answering_all());
        harness.record_attempt("Vitoshka", PROTOCOL, false);
        harness.record_attempt(NICKNAME, PROTOCOL, true);
        harness.save_configuration(&[], &[PROTOCOL]);

        harness.finder.find(&StopSignal::new());

        assert!(harness.event_texts().contains(
            &"location order (from Settings, by recent success): Khalifa, Vitoshka".to_string()
        ));
        assert_eq!(harness.cli.connect_calls()[0], "connect Khalifa ikev2");
    }

    #[test]
    fn should_try_protocols_in_the_configured_order() {
        let harness = Harness::new();
        harness.record_attempt(NICKNAME, "wireguard", true);
        harness.record_attempt(NICKNAME, "wireguard", true);
        harness.save_configuration(&[CITY], &["ikev2", "wireguard"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls()[0], "connect Khalifa ikev2");
        let texts = harness.event_texts();
        assert!(texts.contains(&"protocol order (from Settings): ikev2, wireguard".to_string()));
        assert!(texts
            .contains(&"location order (from Settings, by recent success): Khalifa".to_string()));
    }

    #[test]
    fn should_report_already_running_when_locked() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);
        let second_outcome = Arc::new(Mutex::new(None));
        let finder = harness.finder.clone();
        let recorded = second_outcome.clone();
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| {
                *recorded.lock().unwrap() = Some(finder.find(&StopSignal::new()));
            }));
        });

        harness.finder.find(&StopSignal::new());

        assert_eq!(
            *second_outcome.lock().unwrap(),
            Some(Outcome::AlreadyRunning)
        );
        assert!(harness
            .event_texts()
            .contains(&"search already running".to_string()));
    }

    #[test]
    fn should_say_why_when_a_folder_blocks_the_lock() {
        let harness = Harness::new();
        let lock = harness.paths.lock_file();
        fs::create_dir_all(&lock).unwrap();
        fs::write(lock.join("keep"), "").unwrap();

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Failed);
        assert_eq!(
            harness.event_texts(),
            [format!(
                "{} is a folder with files in it; remove it so a search can start",
                lock.display()
            )]
        );
        assert!(harness.cli.connect_calls().is_empty());
    }

    #[test]
    fn should_report_a_failed_attempt_after_logging_it() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, protocol| protocol == "tcp"));
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);

        harness.finder.find(&StopSignal::new());

        let pair_lines: Vec<String> = harness
            .event_texts()
            .into_iter()
            .filter(|text| {
                ["trying: ", "failed: ", "connected: "]
                    .iter()
                    .any(|prefix| text.starts_with(prefix))
            })
            .collect();
        // The result lines carry the port the status reports: none after a failed connect, 500 from the fake.
        assert_eq!(
            pair_lines,
            [
                "trying: Khalifa / ikev2",
                "failed: Khalifa / ikev2",
                "trying: Khalifa / tcp",
                "connected: Khalifa / tcp:500"
            ]
        );
    }

    #[test]
    fn should_not_log_the_attempt_a_stop_abandons() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);
        let stop = StopSignal::new();
        let requester = stop.clone();
        // Stopped right as the tunnel starts coming up, so the stop has a live tunnel to tear down.
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| requester.request()))
        });

        assert_eq!(harness.finder.find(&stop), Outcome::Stopped);
        assert_eq!(harness.cli.connect_calls().len(), 1);
        assert_eq!(harness.cli.state(), FakeTunnelState::Disconnected);
        assert!(harness.logged_attempts().is_empty());
        assert!(harness
            .event_texts()
            .contains(&"search stopped".to_string()));
    }

    #[test]
    fn should_stop_at_once_when_stopped_during_the_blocked_site_check() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);
        let stop = StopSignal::new();
        let requester = stop.clone();
        harness.probe.set_hangs_after(move || requester.request());
        let started = Instant::now();

        assert_eq!(harness.finder.find(&stop), Outcome::Stopped);
        assert!(started.elapsed() < PROMPT_LIMIT);
        assert!(harness.logged_attempts().is_empty());
        assert_eq!(harness.cli.state(), FakeTunnelState::Disconnected);
    }

    #[test]
    fn should_not_reconnect_after_a_stopped_find() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, _| false));
        harness.save_configuration(&[CITY], &["ikev2", "tcp", "udp"]);
        let stop = StopSignal::new();
        let requester = stop.clone();
        let last_good_writer = harness.attempt_log();
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, protocol| {
                if protocol == "tcp" {
                    last_good_writer.write_last_good(&pair(NICKNAME, "wireguard"));
                    requester.request();
                }
            }));
        });

        harness.finder.find(&stop);

        assert_eq!(
            harness.cli.connect_calls(),
            ["connect Khalifa ikev2", "connect Khalifa tcp"]
        );
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,fail"]);
        assert!(!harness
            .event_texts()
            .contains(&"survey stopped".to_string()));
    }

    #[test]
    fn should_report_when_no_datacenter_matches_the_saved_cities() {
        let harness = Harness::new();
        harness.save_configuration(&["Reykjavik"], &[PROTOCOL]);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Failed);
        assert!(harness.cli.connect_calls().is_empty());
        assert!(harness
            .event_texts()
            .contains(&"no datacenters match the locations in Settings".to_string()));
    }

    #[test]
    fn should_report_when_every_saved_location_is_unknown() {
        let harness = Harness::new();
        harness.save_locations(&["Retired", "Gone"], &[PROTOCOL], true);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Failed);
        assert!(harness.cli.connect_calls().is_empty());
        assert!(harness
            .event_texts()
            .contains(&"no datacenters match the locations in Settings".to_string()));
    }

    #[test]
    fn should_report_nothing_worked_with_an_empty_protocol_order() {
        let harness = Harness::new();
        harness.save_configuration(&[CITY], &[]);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Failed);
        assert!(harness.cli.connect_calls().is_empty());
        assert_eq!(harness.last_event(), EngineEvent::NothingWorked);
    }
}

mod watch {
    use super::*;

    #[test]
    fn should_do_nothing_when_the_tunnel_works() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.state = FakeTunnelState::Connected);

        assert_eq!(harness.finder.watch(&StopSignal::new()), Outcome::Succeeded);
        assert!(harness.cli.connect_calls().is_empty());
        assert!(harness.event_texts().is_empty());
    }

    #[test]
    fn should_search_when_the_tunnel_is_down() {
        let harness = Harness::new();
        harness.save_configuration(&[CITY], &[PROTOCOL]);

        assert_eq!(harness.finder.watch(&StopSignal::new()), Outcome::Succeeded);
        assert_eq!(harness.event_texts()[0], "tunnel down");
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,ok"]);
    }

    #[test]
    fn should_search_when_the_tunnel_is_up_but_the_blocked_site_does_not_load() {
        let harness = Harness::new();
        harness.cli.with(|state| {
            state.state = FakeTunnelState::Connected;
            state.connected_location = "Atlanta - Magic City".to_string();
        });
        harness
            .probe
            .set_loads_through(|location| location == NICKNAME);
        harness.save_configuration(&[CITY], &[PROTOCOL]);

        harness.finder.watch(&StopSignal::new());

        assert_eq!(harness.event_texts()[0], "tunnel down");
        assert_eq!(harness.logged_attempts(), ["Khalifa,ikev2,ok"]);
    }

    #[test]
    fn should_stop_like_a_find_when_stopped() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);
        let stop = StopSignal::new();
        let requester = stop.clone();
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| requester.request()))
        });

        assert_eq!(harness.finder.watch(&stop), Outcome::Stopped);
        assert_eq!(harness.cli.connect_calls().len(), 1);
        assert_eq!(harness.cli.state(), FakeTunnelState::Disconnected);
        assert_eq!(harness.last_event(), EngineEvent::SearchStopped);
    }
}

mod watch_health_check {
    use super::*;

    #[test]
    fn should_leave_the_tunnel_alone_when_stopped_during_the_health_check() {
        let harness = Harness::new();
        harness
            .cli
            .with(|state| state.state = FakeTunnelState::Connected);
        let stop = StopSignal::new();
        let requester = stop.clone();
        harness.probe.set_hangs_after(move || requester.request());

        assert_eq!(harness.finder.watch(&stop), Outcome::Stopped);
        assert_eq!(harness.cli.count_calls(DISCONNECT_CALL), 0);
        assert_eq!(harness.cli.state(), FakeTunnelState::Connected);
        assert!(harness.event_texts().is_empty());
    }

    #[test]
    fn should_disconnect_when_stopped_after_tunnel_down() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, PROTOCOL);
        let stop = StopSignal::new();
        let requester = stop.clone();
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| requester.request()))
        });

        assert_eq!(harness.finder.watch(&stop), Outcome::Stopped);
        assert_eq!(harness.event_texts()[0], "tunnel down");
        assert!(harness.cli.count_calls(DISCONNECT_CALL) >= 1);
        assert_eq!(harness.cli.state(), FakeTunnelState::Disconnected);
    }
}

mod survey {
    use super::*;

    #[test]
    fn should_try_every_pair_without_stopping_at_a_success() {
        let harness = Harness::new();
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);

        harness.finder.survey(&StopSignal::new());

        assert_eq!(
            harness.logged_attempts(),
            ["Khalifa,ikev2,ok", "Khalifa,tcp,ok", "Khalifa,tcp,ok"]
        );
        assert!(harness
            .event_texts()
            .contains(&"survey finished".to_string()));
    }

    #[test]
    fn should_finish_survey_reconnect_despite_a_second_stop() {
        let harness = Harness::new();
        harness.write_last_good(NICKNAME, "wireguard");
        harness
            .cli
            .with(|state| state.pair_connects = Box::new(|_, protocol| protocol == "wireguard"));
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);
        let stop = StopSignal::new();
        let requester = stop.clone();
        // Every connect requests a stop, including the reconnect, which must finish anyway.
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| requester.request()))
        });

        assert_eq!(harness.finder.survey(&stop), Outcome::Stopped);
        assert!(harness
            .event_texts()
            .contains(&"survey stopped".to_string()));
        assert_eq!(
            harness.cli.connect_calls().last().unwrap(),
            "connect Khalifa wireguard"
        );
        assert_eq!(harness.cli.state(), FakeTunnelState::Connected);
        assert_eq!(harness.logged_attempts(), ["Khalifa,wireguard,ok"]);
    }
}

fn never_connects(harness: &Harness) {
    harness
        .cli
        .with(|state| state.pair_connects = Box::new(|_, _| false));
}

mod phases_and_ports {
    use super::*;

    #[test]
    fn should_print_the_phases_in_order() {
        let harness = Harness::new();
        never_connects(&harness);
        harness.write_last_good(NICKNAME, PROTOCOL);
        harness.set_ports("ikev2", "500");
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(
            harness.phases(),
            [
                "phase: last pair that worked",
                "phase: reachability",
                "phase: ikev2 on port 500",
                "phase: tcp on port 443",
                "phase: tcp on port 587",
            ]
        );
    }

    #[test]
    fn should_try_the_preferred_port_first() {
        let harness = Harness::new();
        harness.set_ports("tcp", "443, 587");
        harness.write_log("2026-10-22T14:00:00,Khalifa,tcp,ok,587\n");
        harness.save_configuration(&[CITY], &["tcp"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls()[0], "connect Khalifa tcp:587");
    }

    #[test]
    fn should_try_other_ports_only_after_the_first_pass_fails() {
        let harness = Harness::new();
        never_connects(&harness);
        harness.set_ports("ikev2", "500");
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["ikev2", "tcp"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(
            harness.cli.connect_calls(),
            [
                "connect Khalifa ikev2:500",
                "connect Khalifa tcp:443",
                "connect Khalifa tcp:587"
            ]
        );
        assert_eq!(harness.last_event(), EngineEvent::NothingWorked);
    }

    #[test]
    fn should_skip_the_second_pass_when_the_first_pass_connects() {
        let harness = Harness::new();
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["tcp"]);

        assert_eq!(harness.finder.find(&StopSignal::new()), Outcome::Succeeded);
        assert_eq!(harness.cli.connect_calls(), ["connect Khalifa tcp:443"]);
        assert!(harness
            .event_texts()
            .contains(&"connected: Khalifa / tcp:443".to_string()));
    }

    #[test]
    fn should_count_four_column_history_for_the_default_port() {
        let harness = Harness::new();
        harness.set_ports("tcp", "443, 587");
        harness.write_log(
            "2026-10-22T14:00:00,Khalifa,tcp,ok\n2026-10-22T14:01:00,Khalifa,tcp,fail,587\n",
        );
        harness.save_configuration(&[CITY], &["tcp"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls()[0], "connect Khalifa tcp:443");
    }

    #[test]
    fn should_log_the_status_port_when_the_cli_lists_no_ports() {
        let harness = Harness::new();
        harness.save_configuration(&[CITY], &[PROTOCOL]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls(), ["connect Khalifa ikev2"]);
        let log = fs::read_to_string(harness.paths.log_file()).unwrap();
        assert!(log.ends_with(",Khalifa,ikev2,ok,500\n"), "{log}");
    }

    #[test]
    fn should_survey_every_port() {
        let harness = Harness::new();
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["tcp"]);

        harness.finder.survey(&StopSignal::new());

        let calls = harness.cli.connect_calls();
        assert!(
            calls.contains(&"connect Khalifa tcp:443".to_string()),
            "{calls:?}"
        );
        assert!(
            calls.contains(&"connect Khalifa tcp:587".to_string()),
            "{calls:?}"
        );
        assert!(harness
            .phases()
            .contains(&"phase: tcp on port 587".to_string()));
    }

    #[test]
    fn should_reconnect_last_good_on_its_port_after_a_stopped_survey() {
        let harness = Harness::new();
        harness.attempt_log().write_last_good(&ServerPair {
            port: Some(587),
            ..pair(NICKNAME, "tcp")
        });
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["tcp"]);
        let stop = StopSignal::new();
        let requester = stop.clone();
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| requester.request()))
        });

        assert_eq!(harness.finder.survey(&stop), Outcome::Stopped);
        assert_eq!(
            harness.cli.connect_calls().last().unwrap(),
            "connect Khalifa tcp:587"
        );
    }

    #[test]
    fn should_skip_other_ports_when_no_address_answered() {
        let harness = Harness::with(vec![], FakeReachabilityProbe::answering(&[]));
        never_connects(&harness);
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["tcp"]);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls(), ["connect Khalifa tcp:443"]);
        assert!(harness.event_texts().contains(
            &"no address answered, so the network may be down; skipping the other ports"
                .to_string()
        ));
        assert_eq!(harness.last_event(), EngineEvent::NothingWorked);
    }

    #[test]
    fn should_stop_during_the_second_pass_like_a_find() {
        let harness = Harness::new();
        never_connects(&harness);
        harness.set_ports("tcp", "443, 587");
        harness.save_configuration(&[CITY], &["tcp"]);
        let stop = StopSignal::new();
        let requester = stop.clone();
        let connects = Arc::new(Mutex::new(0));
        harness.cli.with(move |state| {
            state.before_connect = Some(Box::new(move |_, _| {
                let mut count = connects.lock().unwrap();
                *count += 1;
                if *count == 2 {
                    requester.request();
                }
            }));
        });

        assert_eq!(harness.finder.find(&stop), Outcome::Stopped);
        assert_eq!(harness.cli.connect_calls().len(), 2);
        assert_eq!(harness.cli.state(), FakeTunnelState::Disconnected);
        assert!(harness
            .event_texts()
            .contains(&"search stopped".to_string()));
    }
}

mod location_order {
    use super::*;

    fn two_datacenters() -> Harness {
        let harness = Harness::with(vec![sofia()], FakeReachabilityProbe::answering_all());
        harness.record_attempt("Vitoshka", PROTOCOL, true);
        harness.record_attempt(NICKNAME, PROTOCOL, false);
        harness
    }

    #[test]
    fn should_order_locations_from_settings_when_ranking_is_off() {
        let harness = two_datacenters();
        harness.save_locations(&[NICKNAME, "Vitoshka"], &[PROTOCOL], false);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls()[0], "connect Khalifa ikev2");
        assert!(harness
            .event_texts()
            .contains(&"location order (from Settings): Khalifa, Vitoshka".to_string()));
    }

    #[test]
    fn should_rank_locations_when_ranking_is_on() {
        let harness = two_datacenters();
        harness.save_locations(&[NICKNAME, "Vitoshka"], &[PROTOCOL], true);

        harness.finder.find(&StopSignal::new());

        assert_eq!(harness.cli.connect_calls()[0], "connect Vitoshka ikev2");
        assert!(harness.event_texts().contains(
            &"location order (from Settings, by recent success): Vitoshka, Khalifa".to_string()
        ));
    }

    // The anti-goal: an untouched install, with no history, tries exactly the pairs it tried before.
    #[test]
    fn should_search_todays_pairs_first_with_default_settings() {
        let harness = Harness::with_bundled(
            nasib_engine::catalog::bundled(),
            FakeReachabilityProbe::answering_all(),
        );
        never_connects(&harness);

        harness.finder.find(&StopSignal::new());

        let expected: Vec<String> = ALL_PROTOCOLS
            .iter()
            .flat_map(|protocol| {
                DEFAULT_LOCATIONS
                    .iter()
                    .map(move |nickname| format!("connect {nickname} {protocol}"))
            })
            .collect();
        assert_eq!(harness.cli.connect_calls(), expected);
    }
}
