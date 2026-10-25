mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use nasib_linux::engine_process::{engine_beside, EngineProcess};
use support::TemporaryDirectory;

const LINE_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

fn stub_engine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/stub-engine")
}

/// Starts the stub and returns the channels its lines and exit code arrive on.
fn start(arguments: &[&str]) -> (EngineProcess, mpsc::Receiver<String>, mpsc::Receiver<i32>) {
    let (line_sender, lines) = mpsc::channel();
    let (exit_sender, exit) = mpsc::channel();
    let process = EngineProcess::start(
        &stub_engine(),
        arguments,
        move |line| {
            let _ = line_sender.send(line);
        },
        move |code| {
            let _ = exit_sender.send(code);
        },
    )
    .unwrap();
    (process, lines, exit)
}

mod start {
    use super::*;

    #[test]
    fn should_read_lines_until_exit() {
        let (_process, lines, exit) = start(&["lines"]);

        assert_eq!(exit.recv_timeout(LINE_TIMEOUT), Ok(0));
        let received: Vec<String> = lines.try_iter().collect();
        assert_eq!(
            received,
            [
                "phase: tcp on port 443",
                "trying: Lygos / tcp:443",
                "connected: Lygos / tcp:443"
            ]
        );
    }

    #[test]
    fn should_report_an_engine_that_cannot_start() {
        let result =
            EngineProcess::start(Path::new("/nonexistent/nasib-engine"), &[], |_| {}, |_| {});

        assert!(result.is_err());
    }
}

mod stop {
    use super::*;

    #[test]
    fn should_send_stop_on_standard_input() {
        let (mut process, lines, exit) = start(&["stop"]);
        assert_eq!(lines.recv_timeout(LINE_TIMEOUT).as_deref(), Ok("ready"));

        process.stop();

        assert_eq!(lines.recv_timeout(LINE_TIMEOUT).as_deref(), Ok("stopped"));
        assert_eq!(exit.recv_timeout(LINE_TIMEOUT), Ok(130));
    }

    // Quitting the app mid-survey must leave the engine to finish reconnecting on its own.
    #[test]
    fn should_close_input_without_killing_on_drop() {
        let directory = TemporaryDirectory::new();
        let marker = directory.path().join("closed");
        let (process, lines, _exit) = start(&["input-closed", marker.to_str().unwrap()]);
        assert_eq!(lines.recv_timeout(LINE_TIMEOUT).as_deref(), Ok("ready"));

        drop(process);

        let deadline = Instant::now() + LINE_TIMEOUT;
        while !marker.exists() && Instant::now() < deadline {
            thread::sleep(POLL_INTERVAL);
        }
        assert_eq!(fs::read_to_string(&marker).unwrap().trim(), "closed");
    }
}

mod run_once {
    use super::*;

    #[test]
    fn should_return_the_trimmed_output() {
        assert_eq!(
            EngineProcess::run_once(&stub_engine(), "once"),
            "connected: Lygos / tcp:443"
        );
    }

    #[test]
    fn should_return_nothing_when_the_engine_cannot_start() {
        assert_eq!(
            EngineProcess::run_once(Path::new("/nonexistent/nasib-engine"), "--status"),
            ""
        );
    }
}

mod engine_path {
    use super::*;

    fn write_file(path: &Path, mode: u32) {
        fs::write(path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    #[test]
    fn should_find_the_engine_next_to_the_app() {
        let directory = TemporaryDirectory::new();
        write_file(&directory.path().join("nasib-engine"), 0o755);

        assert_eq!(
            engine_beside(&directory.path().join("nasib")),
            Some(directory.path().join("nasib-engine"))
        );
    }

    #[test]
    fn should_report_no_engine_when_the_file_is_missing() {
        let directory = TemporaryDirectory::new();

        assert_eq!(engine_beside(&directory.path().join("nasib")), None);
    }

    #[test]
    fn should_report_no_engine_when_the_file_is_not_executable() {
        let directory = TemporaryDirectory::new();
        write_file(&directory.path().join("nasib-engine"), 0o644);

        assert_eq!(engine_beside(&directory.path().join("nasib")), None);
    }
}
