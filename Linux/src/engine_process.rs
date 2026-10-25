//! Runs the bundled `nasib-engine` and hands its output lines to the app.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const ENGINE_NAME: &str = "nasib-engine";
const STOP_LINE: &[u8] = b"stop\n";
/// One-shot modes answer in a second or two; this only guards against a CLI that hangs.
const RUN_ONCE_TIMEOUT: Duration = Duration::from_secs(30);
const RUN_ONCE_POLL_INTERVAL: Duration = Duration::from_millis(50);
/// The exit code reported when the engine ended without one, such as from a signal.
const UNKNOWN_EXIT_CODE: i32 = -1;
const EXECUTABLE_BITS: u32 = 0o111;

/// The engine next to the given app executable, when it's there and executable.
pub fn engine_beside(executable: &Path) -> Option<PathBuf> {
    let engine = executable.parent()?.join(ENGINE_NAME);
    let metadata = engine.metadata().ok()?;
    (metadata.is_file() && metadata.permissions().mode() & EXECUTABLE_BITS != 0).then_some(engine)
}

/// The engine installed next to the running app.
pub fn engine_path() -> Option<PathBuf> {
    engine_beside(&std::env::current_exe().ok()?)
}

/// A running engine. Its standard input stays open for the whole run, since the engine treats it closing
/// as a stop.
pub struct EngineProcess {
    input: Option<ChildStdin>,
}

/// Sends every line from the stream to the callback until the stream ends.
fn forward_lines(
    stream: impl Read + Send + 'static,
    on_line: Arc<dyn Fn(String) + Send + Sync>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(stream).lines() {
            match line {
                Ok(line) => on_line(line),
                Err(_) => break,
            }
        }
    })
}

/// Waits for the engine's output to end, then for the engine itself, so the exit comes after every line.
fn wait_for_exit(
    mut child: Child,
    readers: Vec<thread::JoinHandle<()>>,
    on_exit: impl FnOnce(i32) + Send + 'static,
) {
    thread::spawn(move || {
        for reader in readers {
            let _ = reader.join();
        }
        let code = child
            .wait()
            .ok()
            .and_then(|status| status.code())
            .unwrap_or(UNKNOWN_EXIT_CODE);
        on_exit(code);
    });
}

impl EngineProcess {
    /// Starts the engine. Lines and the exit code arrive on background threads.
    pub fn start(
        engine: &Path,
        arguments: &[&str],
        on_line: impl Fn(String) + Send + Sync + 'static,
        on_exit: impl FnOnce(i32) + Send + 'static,
    ) -> io::Result<EngineProcess> {
        // Some environments, such as an emulated x64 container, report a missing program as started, so
        // the file is checked first.
        if !engine.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                engine.display().to_string(),
            ));
        }
        let mut child = Command::new(engine)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let on_line: Arc<dyn Fn(String) + Send + Sync> = Arc::new(on_line);
        let mut readers = Vec::new();
        if let Some(output) = child.stdout.take() {
            readers.push(forward_lines(output, on_line.clone()));
        }
        if let Some(errors) = child.stderr.take() {
            readers.push(forward_lines(errors, on_line));
        }
        let input = child.stdin.take();
        wait_for_exit(child, readers, on_exit);
        Ok(EngineProcess { input })
    }

    /// Asks the engine to stop. A stopped find disconnects; a stopped survey reconnects the last server
    /// that worked.
    pub fn stop(&mut self) {
        if let Some(input) = self.input.as_mut() {
            let _ = input.write_all(STOP_LINE).and_then(|()| input.flush());
        }
    }

    /// Runs a one-shot mode such as `--status` and returns what it prints, trimmed; empty when the engine
    /// can't run or takes too long. Call it off the main thread.
    pub fn run_once(engine: &Path, argument: &str) -> String {
        let Ok(mut child) = Command::new(engine)
            .arg(argument)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return String::new();
        };
        let reader = child.stdout.take().map(|mut output| {
            thread::spawn(move || {
                let mut text = String::new();
                let _ = output.read_to_string(&mut text);
                text
            })
        });
        let deadline = Instant::now() + RUN_ONCE_TIMEOUT;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => thread::sleep(RUN_ONCE_POLL_INTERVAL),
                // A one-shot mode only reads, so ending it early changes nothing.
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return String::new();
                }
            }
        }
        reader
            .and_then(|reader| reader.join().ok())
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    }
}

impl Drop for EngineProcess {
    /// Closes the engine's input and never kills it: quitting mid-survey leaves the engine to reconnect the
    /// last server that worked and exit on its own.
    fn drop(&mut self) {
        self.input.take();
    }
}
