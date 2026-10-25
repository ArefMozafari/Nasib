use std::collections::HashSet;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nasib_engine::datacenter::Datacenter;
use nasib_engine::reachability::{self, ReachabilityProbe, TcpReachabilityProbe, REACH_PORT};
use nasib_engine::stop::{StopSignal, Stopped};

const REACH_TIMEOUT: Duration = Duration::from_millis(500);
const SLOW_KNOCK: Duration = Duration::from_secs(10);
const REQUEST_DELAY: Duration = Duration::from_millis(50);
const PROMPT_LIMIT: Duration = Duration::from_millis(500);

/// Answers for the listed addresses only, and remembers every address it was asked about.
struct FakeProbe {
    answering: HashSet<String>,
    asked: Mutex<Vec<String>>,
    delay: Duration,
}

impl FakeProbe {
    fn answering(addresses: &[&str]) -> Arc<FakeProbe> {
        Arc::new(FakeProbe {
            answering: addresses
                .iter()
                .map(|address| address.to_string())
                .collect(),
            asked: Mutex::new(Vec::new()),
            delay: Duration::ZERO,
        })
    }
}

impl ReachabilityProbe for FakeProbe {
    fn answers(&self, address: &str) -> bool {
        self.asked.lock().unwrap().push(address.to_string());
        thread::sleep(self.delay);
        self.answering.contains(address)
    }
}

fn datacenter(location_name: &str, addresses: &[&str]) -> Datacenter {
    Datacenter::from_location_name(
        location_name,
        addresses
            .iter()
            .map(|address| address.to_string())
            .collect(),
    )
    .unwrap()
}

mod reachable {
    use super::*;

    #[test]
    fn should_keep_datacenters_with_at_least_one_answering_address_in_the_given_order() {
        let datacenters = vec![
            datacenter("Dubai - Khalifa", &["203.0.113.1", "203.0.113.2"]),
            datacenter("Istanbul - Lygos", &["203.0.113.3"]),
            datacenter("Sofia - Vitoshka", &["203.0.113.4"]),
        ];
        let probe = FakeProbe::answering(&["203.0.113.2", "203.0.113.4"]);

        let reachable = reachability::reachable(&datacenters, probe, &StopSignal::new()).unwrap();

        let nicknames: Vec<&str> = reachable
            .iter()
            .map(|datacenter| datacenter.nickname.as_str())
            .collect();
        assert_eq!(nicknames, ["Khalifa", "Vitoshka"]);
    }

    #[test]
    fn should_knock_on_every_address() {
        let datacenters = vec![
            datacenter("Dubai - Khalifa", &["203.0.113.1", "203.0.113.2"]),
            datacenter("Istanbul - Lygos", &["203.0.113.3"]),
        ];
        let probe = FakeProbe::answering(&[]);

        reachability::reachable(&datacenters, probe.clone(), &StopSignal::new()).unwrap();

        let asked: HashSet<String> = probe.asked.lock().unwrap().iter().cloned().collect();
        assert_eq!(
            asked,
            ["203.0.113.1", "203.0.113.2", "203.0.113.3"]
                .map(String::from)
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn should_return_stopped_while_knocks_are_running() {
        let datacenters = vec![datacenter("Dubai - Khalifa", &["203.0.113.1"])];
        let probe = Arc::new(FakeProbe {
            answering: HashSet::new(),
            asked: Mutex::new(Vec::new()),
            delay: SLOW_KNOCK,
        });
        let stop = StopSignal::new();
        let requester = stop.clone();
        let started = Instant::now();
        thread::spawn(move || {
            thread::sleep(REQUEST_DELAY);
            requester.request();
        });

        assert_eq!(
            reachability::reachable(&datacenters, probe, &stop),
            Err(Stopped)
        );
        assert!(started.elapsed() < PROMPT_LIMIT);
    }
}

mod tcp_probe {
    use super::*;

    #[test]
    fn should_knock_on_port_443() {
        assert_eq!(REACH_PORT, 443);
    }

    #[test]
    fn should_answer_when_something_listens_on_the_port() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        assert!(TcpReachabilityProbe::new(port, REACH_TIMEOUT).answers("127.0.0.1"));
    }

    #[test]
    fn should_not_answer_when_nothing_listens_on_the_port() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        assert!(!TcpReachabilityProbe::new(port, REACH_TIMEOUT).answers("127.0.0.1"));
    }

    #[test]
    fn should_not_answer_for_something_that_is_not_an_address() {
        assert!(!TcpReachabilityProbe::new(REACH_PORT, REACH_TIMEOUT).answers("not-an-address"));
    }
}
