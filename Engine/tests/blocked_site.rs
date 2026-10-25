use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use nasib_engine::blocked_site::{BlockedSiteProbe, HttpBlockedSiteProbe};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const SHORT_TIMEOUT: Duration = Duration::from_millis(200);
const TIMEOUT_LIMIT: Duration = Duration::from_secs(2);
const NO_CONTENT_RESPONSE: &str = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
const BLOCK_PAGE_RESPONSE: &str =
    "HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nblocked";

/// Answers every request with the given response, and returns the URL to request.
fn responder(response: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{address}/generate_204")
}

/// Accepts connections and never answers, like a filtered network.
fn silent_listener() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        let mut held = Vec::new();
        for stream in listener.incoming().flatten() {
            held.push(stream);
        }
    });
    format!("http://{address}/generate_204")
}

mod loads {
    use super::*;

    #[test]
    fn should_count_a_204_answer_as_loaded() {
        let probe = HttpBlockedSiteProbe::new(responder(NO_CONTENT_RESPONSE), PROBE_TIMEOUT);

        assert!(probe.loads());
    }

    #[test]
    fn should_not_count_any_other_answer_such_as_a_block_page() {
        let probe = HttpBlockedSiteProbe::new(responder(BLOCK_PAGE_RESPONSE), PROBE_TIMEOUT);

        assert!(!probe.loads());
    }

    #[test]
    fn should_not_count_a_request_that_times_out() {
        let probe = HttpBlockedSiteProbe::new(silent_listener(), SHORT_TIMEOUT);
        let started = Instant::now();

        assert!(!probe.loads());
        assert!(started.elapsed() < TIMEOUT_LIMIT);
    }

    #[test]
    fn should_not_count_a_refused_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let probe =
            HttpBlockedSiteProbe::new(format!("http://{address}/generate_204"), PROBE_TIMEOUT);

        assert!(!probe.loads());
    }

    #[test]
    fn should_ignore_a_system_proxy() {
        let dead_proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy_address = format!("http://{}", dead_proxy.local_addr().unwrap());
        drop(dead_proxy);
        // Only this test file sets these, and every probe here ignores them by design.
        unsafe {
            std::env::set_var("HTTP_PROXY", &proxy_address);
            std::env::set_var("HTTPS_PROXY", &proxy_address);
            std::env::set_var("ALL_PROXY", &proxy_address);
        }
        let probe = HttpBlockedSiteProbe::new(responder(NO_CONTENT_RESPONSE), PROBE_TIMEOUT);

        assert!(probe.loads());
    }
}
