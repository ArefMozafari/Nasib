//! Runs the built binary against the fake windscribe-cli script in a throwaway state folder, the way the
//! apps run it. The fake is a POSIX shell script, so these tests run on macOS and Linux.
#![cfg(unix)]

mod support;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use support::TemporaryDirectory;

const ENGINE: &str = env!("CARGO_BIN_EXE_nasib-engine");
const FAKE_CLI_FILE: &str = "tests/support/fake-windscribe-cli";
const FAST_TIME_SCALE: &str = "100";
/// The real timings, for the tests that hold the engine to how long a stop may take.
const REAL_TIME_SCALE: &str = "1";
const LINE_WAIT: Duration = Duration::from_secs(10);
/// ws-find.sh's own test allowed six seconds from launch for a stop during the blocked-site check.
const STOP_LIMIT: Duration = Duration::from_secs(6);
const EXIT_WAIT: Duration = Duration::from_secs(20);
const EXIT_POLL: Duration = Duration::from_millis(20);
const STOPPED_EXIT_CODE: i32 = 130;
const CLI_MISSING_EXIT_CODE: i32 = 2;
const NO_CONTENT_RESPONSE: &str = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";

/// A sandbox with one datacenter on 127.0.0.1, so no test knocks on a real server.
struct Sandbox {
    directory: TemporaryDirectory,
    probe_url: String,
}

/// Answers every request with 204, and returns the URL to request.
fn no_content_responder() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(NO_CONTENT_RESPONSE.as_bytes());
        }
    });
    format!("http://{address}/generate_204")
}

/// Accepts connections and never answers, like a blocked-site check on a dead network.
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

impl Sandbox {
    fn new(probe_url: String) -> Sandbox {
        let directory = TemporaryDirectory::new();
        let sandbox = Sandbox {
            directory,
            probe_url,
        };
        fs::create_dir_all(sandbox.state_directory()).unwrap();
        fs::create_dir_all(sandbox.log_directory()).unwrap();
        fs::create_dir_all(sandbox.cli_directory()).unwrap();
        fs::write(
            sandbox.log_directory().join("client.log"),
            "{\"msg\": \"MutableLocationInfo created:  \\\"Testville - Fake\\\" \\\"127.0.0.1; \\\" ; Selected node: 1\"}\n",
        )
        .unwrap();
        fs::write(
            sandbox.state_directory().join("config.json"),
            r#"{ "cities": ["Testville"], "protocolOrder": ["ikev2"] }"#,
        )
        .unwrap();
        sandbox
    }

    fn state_directory(&self) -> PathBuf {
        self.directory.path().join("state")
    }

    fn log_directory(&self) -> PathBuf {
        self.directory.path().join("logs")
    }

    fn cli_directory(&self) -> PathBuf {
        self.directory.path().join("cli")
    }

    fn set_tunnel(&self, state: &str) {
        fs::write(self.cli_directory().join("state"), format!("{state}\n")).unwrap();
    }

    fn tunnel(&self) -> String {
        fs::read_to_string(self.cli_directory().join("state"))
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.cli_directory().join("calls"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    fn list_ports(&self, protocol: &str, ports: &str) {
        fs::write(
            self.cli_directory().join(format!("ports-{protocol}")),
            format!("{ports}\n"),
        )
        .unwrap();
    }

    fn never_connect(&self) {
        fs::write(self.cli_directory().join("never_connects"), "").unwrap();
    }

    fn logged_attempts(&self) -> String {
        fs::read_to_string(self.state_directory().join("log.csv")).unwrap_or_default()
    }

    fn command(&self, arguments: &[&str], time_scale: &str, cli: &Path) -> Command {
        let mut command = Command::new(ENGINE);
        command
            .args(arguments)
            .env("NASIB_STATE_DIR", self.state_directory())
            .env("NASIB_WINDSCRIBE_LOG_DIR", self.log_directory())
            .env("NASIB_WINDSCRIBE_CLI", cli)
            .env("NASIB_PROBE_URL", &self.probe_url)
            .env("NASIB_TIME_SCALE", time_scale)
            .env("FAKE_CLI_DIR", self.cli_directory())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        command
    }

    fn fake_cli() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_CLI_FILE)
    }

    /// Runs the engine to the end with standard input held open, and returns its exit code and output.
    fn run(&self, arguments: &[&str]) -> (i32, String) {
        let mut child = self
            .command(arguments, FAST_TIME_SCALE, &Sandbox::fake_cli())
            .spawn()
            .unwrap();
        let _held_input = child.stdin.take();
        let mut output = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();
        (child.wait().unwrap().code().unwrap_or(-1), output)
    }

    /// Starts a find on the real timings, and returns it with its output lines as they arrive.
    fn start_find(&self) -> (Child, Receiver<String>) {
        let mut child = self
            .command(&[], REAL_TIME_SCALE, &Sandbox::fake_cli())
            .spawn()
            .unwrap();
        let output = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(output).lines().map_while(Result::ok) {
                let _ = sender.send(line);
            }
        });
        (child, receiver)
    }
}

/// Collects output lines until one starts with the prefix, which must arrive in time.
fn wait_for_line(receiver: &Receiver<String>, prefix: &str, seen: &mut Vec<String>) {
    let deadline = Instant::now() + LINE_WAIT;
    while Instant::now() < deadline {
        if let Ok(line) = receiver.recv_timeout(EXIT_POLL) {
            let found = line.starts_with(prefix);
            seen.push(line);
            if found {
                return;
            }
        }
    }
    panic!("no line starting with {prefix:?}; saw {seen:?}");
}

/// Waits for the engine to exit, and returns its exit code with the rest of its output.
fn wait_for_exit(child: &mut Child, receiver: &Receiver<String>, seen: &mut Vec<String>) -> i32 {
    let deadline = Instant::now() + EXIT_WAIT;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            seen.extend(receiver.try_iter());
            thread::sleep(EXIT_POLL);
            seen.extend(receiver.try_iter());
            return status.code().unwrap_or(-1);
        }
        seen.extend(receiver.try_iter());
        thread::sleep(EXIT_POLL);
    }
    let _ = child.kill();
    panic!("the engine did not exit; saw {seen:?}");
}

fn send_interrupt(child: &Child) {
    Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
}

/// Starts a find that hangs in the blocked-site check, stops it the given way, and checks that it
/// stopped in time, disconnected, without logging the abandoned attempt.
fn expect_prompt_stop(stop: impl FnOnce(&mut Child)) {
    let sandbox = Sandbox::new(silent_listener());
    sandbox.set_tunnel("disconnected");
    let started = Instant::now();
    let (mut child, receiver) = sandbox.start_find();
    let mut seen = Vec::new();
    wait_for_line(&receiver, "trying: ", &mut seen);
    // Connect lands after one poll and the check starts after the two-second settle, so at four seconds
    // the engine is inside the eight-second check.
    thread::sleep(Duration::from_secs(4).saturating_sub(started.elapsed()));

    stop(&mut child);
    let code = wait_for_exit(&mut child, &receiver, &mut seen);

    assert_eq!(code, STOPPED_EXIT_CODE, "{seen:?}");
    assert!(
        started.elapsed() <= STOP_LIMIT,
        "took {:?}",
        started.elapsed()
    );
    assert!(seen.contains(&"search stopped".to_string()), "{seen:?}");
    assert_eq!(sandbox.logged_attempts(), "");
    assert_eq!(sandbox.tunnel(), "disconnected");
}

mod cli_check {
    use super::*;

    #[test]
    fn should_print_cli_missing_and_exit_two() {
        let sandbox = Sandbox::new(no_content_responder());
        let missing = sandbox.directory.path().join("missing-cli");
        let mut child = sandbox
            .command(&["--status"], FAST_TIME_SCALE, &missing)
            .spawn()
            .unwrap();
        let mut output = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();

        assert_eq!(child.wait().unwrap().code(), Some(CLI_MISSING_EXIT_CODE));
        assert_eq!(
            output,
            format!("windscribe-cli missing: {}\n", missing.display())
        );
    }

    #[test]
    fn should_print_the_catalog_without_the_cli() {
        let sandbox = Sandbox::new(no_content_responder());
        let missing = sandbox.directory.path().join("missing-cli");
        let mut child = sandbox
            .command(&["--catalog"], FAST_TIME_SCALE, &missing)
            .spawn()
            .unwrap();
        let mut output = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();

        assert_eq!(child.wait().unwrap().code(), Some(0));
        let catalog: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        let regions = catalog["regions"].as_array().unwrap();
        assert!(regions.len() > 1);
        assert_eq!(regions.last().unwrap()["name"], "Other");
        assert_eq!(
            regions.last().unwrap()["datacenters"][0]["nickname"],
            "Fake"
        );
    }
}

mod status {
    use super::*;

    #[test]
    fn should_print_status_line() {
        let sandbox = Sandbox::new(no_content_responder());
        sandbox.set_tunnel("connected");

        assert_eq!(
            sandbox.run(&["--status"]),
            (0, "connected: Testville - Fake / IKEv2\n".to_string())
        );
    }

    #[test]
    fn should_print_down_when_disconnected() {
        let sandbox = Sandbox::new(no_content_responder());

        assert_eq!(sandbox.run(&["--status"]), (0, "down\n".to_string()));
    }
}

mod watch {
    use super::*;

    #[test]
    fn should_exit_zero_on_a_healthy_watch() {
        let sandbox = Sandbox::new(no_content_responder());
        sandbox.set_tunnel("connected");

        assert_eq!(sandbox.run(&["--watch"]), (0, String::new()));
    }

    #[test]
    fn should_search_when_the_tunnel_is_down() {
        let sandbox = Sandbox::new(no_content_responder());

        let (code, output) = sandbox.run(&["--watch"]);

        assert_eq!(code, 0);
        assert!(output.starts_with("tunnel down\n"), "{output}");
        assert!(output.contains("connected: Fake / ikev2:500\n"), "{output}");
    }
}

mod find {
    use super::*;

    #[test]
    fn should_log_and_remember_the_pair_that_works() {
        let sandbox = Sandbox::new(no_content_responder());

        let (code, output) = sandbox.run(&[]);

        assert_eq!(code, 0, "{output}");
        // No ports are listed here, so the attempt logs the port the status reports.
        assert!(
            sandbox.logged_attempts().ends_with(",Fake,ikev2,ok,500\n"),
            "{}",
            sandbox.logged_attempts()
        );
        assert_eq!(
            fs::read_to_string(sandbox.state_directory().join("last_good")).unwrap(),
            "Fake|ikev2:500\n"
        );
    }

    #[test]
    fn should_print_the_port_and_phases() {
        let sandbox = Sandbox::new(no_content_responder());
        sandbox.list_ports("ikev2", "500");

        let (code, output) = sandbox.run(&[]);

        assert_eq!(code, 0, "{output}");
        assert!(output.contains("phase: reachability\n"), "{output}");
        assert!(output.contains("phase: ikev2 on port 500\n"), "{output}");
        assert!(output.contains("trying: Fake / ikev2:500\n"), "{output}");
        assert!(sandbox
            .calls()
            .contains(&"connect -n Fake ikev2:500".to_string()));
    }

    // The sandbox's datacenter doesn't answer knocks, which reads as offline, so a find skips the other
    // ports; a survey still covers them. The second pass after a failed first pass is a finder unit test.
    #[test]
    fn should_survey_the_other_ports() {
        let sandbox = Sandbox::new(no_content_responder());
        sandbox.never_connect();
        sandbox.list_ports("ikev2", "500, 4500");

        let (code, output) = sandbox.run(&["--survey"]);

        assert_eq!(code, 1, "{output}");
        assert!(output.contains("phase: ikev2 on port 4500\n"), "{output}");
        assert!(sandbox
            .logged_attempts()
            .contains(",Fake,ikev2,fail,4500\n"));
    }

    #[test]
    fn should_print_failed_line_and_exit_one_when_nothing_works() {
        let sandbox = Sandbox::new(no_content_responder());
        sandbox.never_connect();

        let (code, output) = sandbox.run(&[]);

        assert_eq!(code, 1);
        assert!(output.contains("failed: Fake / ikev2\n"), "{output}");
        assert!(output.ends_with("nothing worked\n"), "{output}");
    }
}

mod listing {
    use super::*;

    #[test]
    fn should_print_each_datacenter_with_its_addresses() {
        let sandbox = Sandbox::new(no_content_responder());

        assert_eq!(
            sandbox.run(&["--locations"]),
            (0, "Fake: 127.0.0.1\n".to_string())
        );
    }

    #[test]
    fn should_print_the_protocol_and_location_order() {
        let sandbox = Sandbox::new(no_content_responder());

        assert_eq!(
            sandbox.run(&["--rank"]),
            (
                0,
                "protocols (from Settings):\nikev2\nlocations (by recent success):\nFake\n"
                    .to_string()
            )
        );
    }
}

mod stop {
    use super::*;

    #[test]
    fn should_stop_a_find_promptly_on_sigint() {
        expect_prompt_stop(|child| send_interrupt(child));
    }

    #[test]
    fn should_stop_on_a_stop_line() {
        expect_prompt_stop(|child| {
            let mut input = child.stdin.take().unwrap();
            input.write_all(b"stop\n").unwrap();
            // Held open, so only the line can be what stops it.
            std::mem::forget(input);
        });
    }

    #[test]
    fn should_stop_when_standard_input_closes() {
        expect_prompt_stop(|child| drop(child.stdin.take()));
    }
}

/// Starts a run on the real timings that hangs in the blocked-site check, then closes both pipes the way
/// a quitting app does, and returns the exit code and the CLI calls made after that.
fn run_until_the_app_goes_away(sandbox: &Sandbox, arguments: &[&str]) -> (i32, Vec<String>) {
    let started = Instant::now();
    let mut child = sandbox
        .command(arguments, REAL_TIME_SCALE, &Sandbox::fake_cli())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    while !line.starts_with("trying: ") {
        line.clear();
        assert!(
            output.read_line(&mut line).unwrap() > 0,
            "the engine ended before trying a pair"
        );
    }
    // At four seconds the engine is inside the blocked-site check, as in the prompt-stop tests.
    thread::sleep(Duration::from_secs(4).saturating_sub(started.elapsed()));
    let calls_before = sandbox.calls().len();

    drop(output);
    drop(child.stdin.take());
    let deadline = Instant::now() + EXIT_WAIT;
    let code = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status.code().unwrap_or(-1);
        }
        assert!(Instant::now() < deadline, "the engine did not exit");
        thread::sleep(EXIT_POLL);
    };
    (code, sandbox.calls()[calls_before..].to_vec())
}

mod app_goes_away {
    use super::*;

    #[test]
    fn should_disconnect_after_a_find_when_the_app_quits() {
        let sandbox = Sandbox::new(silent_listener());

        let (code, calls) = run_until_the_app_goes_away(&sandbox, &[]);

        assert_eq!(code, STOPPED_EXIT_CODE);
        assert!(calls.contains(&"disconnect".to_string()), "{calls:?}");
        assert_eq!(sandbox.tunnel(), "disconnected");
    }

    #[test]
    fn should_reconnect_last_good_with_its_port_when_the_app_quits_mid_survey() {
        let sandbox = Sandbox::new(silent_listener());
        fs::write(
            sandbox.state_directory().join("last_good"),
            "Fake|ikev2:4500\n",
        )
        .unwrap();

        let (code, calls) = run_until_the_app_goes_away(&sandbox, &["--survey"]);

        assert_eq!(code, STOPPED_EXIT_CODE);
        assert!(
            calls.contains(&"connect -n Fake ikev2:4500".to_string()),
            "{calls:?}"
        );
    }

    #[test]
    fn should_reconnect_the_last_good_pair_after_a_survey_when_the_app_quits() {
        let sandbox = Sandbox::new(silent_listener());
        fs::write(sandbox.state_directory().join("last_good"), "Fake|ikev2\n").unwrap();

        let (code, calls) = run_until_the_app_goes_away(&sandbox, &["--survey"]);

        assert_eq!(code, STOPPED_EXIT_CODE);
        assert!(
            calls.contains(&"connect -n Fake ikev2".to_string()),
            "{calls:?}"
        );
    }
}
