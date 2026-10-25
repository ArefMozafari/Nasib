//! Runs the `windscribe-cli` that ships with the Windscribe app.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const STATUS_ARGUMENTS: [&str; 1] = ["status"];
pub const DISCONNECT_ARGUMENTS: [&str; 1] = ["disconnect"];
const CONNECT_COMMAND: &str = "connect";
const NON_BLOCKING_OPTION: &str = "-n";
const PORTS_COMMAND: &str = "ports";
const PORT_SEPARATOR: char = ':';
const PORT_LIST_SEPARATOR: char = ',';

/// A blocking disconnect normally takes a few seconds; this only guards against a CLI that never exits.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Keeps the CLI from flashing a console window on Windows.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The CLI calls the engine makes. Fakes stand in for it in tests.
pub trait WindscribeCli: Send + Sync {
    /// Returns standard output and error together; empty when the CLI cannot be run.
    fn read_status(&self) -> String;
    /// Connects on the given port, or on whatever port Windscribe picks without one.
    fn connect(&self, nickname: &str, protocol: &str, port: Option<u16>);
    fn disconnect(&self);
    /// Returns what `ports <protocol>` prints: the protocol's ports, default first.
    fn ports(&self, protocol: &str) -> String;
}

pub fn connect_arguments(nickname: &str, protocol: &str, port: Option<u16>) -> [String; 4] {
    let target = match port {
        Some(port) => format!("{protocol}{PORT_SEPARATOR}{port}"),
        None => protocol.to_string(),
    };
    [
        CONNECT_COMMAND.to_string(),
        NON_BLOCKING_OPTION.to_string(),
        nickname.to_string(),
        target,
    ]
}

pub fn ports_arguments(protocol: &str) -> [String; 2] {
    [PORTS_COMMAND.to_string(), protocol.to_string()]
}

/// Reads the comma-separated ports from the first line made only of digits, commas and spaces, which
/// skips the JSON log lines the CLI prints first. Anything else, such as "Windscribe CLI is already
/// running", gives no ports.
pub fn parse_ports(output: &str) -> Vec<u16> {
    let is_port_list = |line: &&str| {
        !line.is_empty()
            && line.chars().all(|character| {
                character.is_ascii_digit() || character == PORT_LIST_SEPARATOR || character == ' '
            })
    };
    let Some(line) = output.lines().map(str::trim).find(is_port_list) else {
        return Vec::new();
    };
    let ports: Option<Vec<u16>> = line
        .split(PORT_LIST_SEPARATOR)
        .map(|port| port.trim().parse().ok())
        .collect();
    ports.unwrap_or_default()
}

pub struct ProcessWindscribeCli {
    path: PathBuf,
}

/// Reads a child's pipe to the end on its own thread, so a full pipe can't stall the child.
fn read_on_thread(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// Waits for the child to exit, killing it after the guard timeout.
fn wait_or_kill(child: &mut Child) {
    let started = Instant::now();
    while started.elapsed() < COMMAND_TIMEOUT {
        if !matches!(child.try_wait(), Ok(None)) {
            return;
        }
        thread::sleep(EXIT_POLL_INTERVAL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

impl ProcessWindscribeCli {
    pub fn new(path: PathBuf) -> ProcessWindscribeCli {
        ProcessWindscribeCli { path }
    }

    pub fn is_installed(&self) -> bool {
        self.path.is_file()
    }

    fn run<S: AsRef<str>>(&self, arguments: &[S]) -> String {
        let mut command = Command::new(&self.path);
        command
            .args(arguments.iter().map(AsRef::as_ref))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let Ok(mut child) = command.spawn() else {
            return String::new();
        };
        let output = read_on_thread(child.stdout.take());
        let error = read_on_thread(child.stderr.take());
        wait_or_kill(&mut child);
        output.join().unwrap_or_default() + &error.join().unwrap_or_default()
    }
}

impl WindscribeCli for ProcessWindscribeCli {
    fn read_status(&self) -> String {
        self.run(&STATUS_ARGUMENTS)
    }

    fn connect(&self, nickname: &str, protocol: &str, port: Option<u16>) {
        self.run(&connect_arguments(nickname, protocol, port));
    }

    fn disconnect(&self) {
        self.run(&DISCONNECT_ARGUMENTS);
    }

    fn ports(&self, protocol: &str) -> String {
        self.run(&ports_arguments(protocol))
    }
}
