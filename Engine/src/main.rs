//! `nasib-engine`: finds a Windscribe datacenter and protocol that works right now, connects to it, and
//! stays connected. The apps run it and read its output lines; see `Engine/README.md`.

use std::env;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

use nasib_engine::attempt_log::{AttemptLog, SystemClock};
use nasib_engine::blocked_site::{HttpBlockedSiteProbe, DEFAULT_PROBE_URL};
use nasib_engine::catalog;
use nasib_engine::catalog_output;
use nasib_engine::configuration::Configuration;
use nasib_engine::events::EngineEvent;
use nasib_engine::finder::{Finder, Outcome};
use nasib_engine::mode::Mode;
use nasib_engine::paths::Paths;
use nasib_engine::ranking::{self, RANK_WINDOW_LINES};
use nasib_engine::reachability::{self, TcpReachabilityProbe, REACH_PORT};
use nasib_engine::stop::StopSignal;
use nasib_engine::timings::Timings;
use nasib_engine::tunnel::Tunnel;
use nasib_engine::tunnel_status::TunnelStatus;
use nasib_engine::windscribe_cli::{ProcessWindscribeCli, WindscribeCli};

/// A test override for the blocked-site page. The apps never set it.
const PROBE_URL_VARIABLE: &str = "NASIB_PROBE_URL";
/// The line an app writes to standard input to stop a run.
const STOP_LINE: &str = "stop";
const USAGE: &str =
    "usage: nasib-engine [--watch | --survey | --reach | --rank | --locations | --status | --catalog]";

const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const CLI_MISSING_EXIT_CODE: u8 = 2;
/// 128 plus SIGINT, as a shell reports a run ended by Ctrl-C.
const STOPPED_EXIT_CODE: u8 = 130;
/// The BSD convention for a command-line usage error.
const USAGE_EXIT_CODE: u8 = 64;

fn print_line(text: &str) {
    // Standard output is line-buffered, so each line reaches the app as soon as it is printed. A write to an
    // app that has quit fails, and is ignored so the clean-up after the stop still runs.
    let _ = writeln!(io::stdout().lock(), "{text}");
}

/// Stops the run on Ctrl-C or SIGTERM, on a `stop` line, or when standard input closes because the app
/// quit. A run never outlives its app by more than its clean-up.
fn listen_for_stop(stop: &StopSignal) {
    let on_signal = stop.clone();
    let _ = ctrlc::set_handler(move || on_signal.request());
    let on_input = stop.clone();
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            match line {
                Ok(line) if line.trim() == STOP_LINE => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        on_input.request();
    });
}

fn exit_code_of(outcome: Outcome) -> u8 {
    match outcome {
        Outcome::Succeeded | Outcome::AlreadyRunning => SUCCESS_EXIT_CODE,
        Outcome::Failed => FAILURE_EXIT_CODE,
        Outcome::Stopped => STOPPED_EXIT_CODE,
    }
}

fn run_search(mode: Mode, paths: Paths, cli: Arc<dyn WindscribeCli>, timings: Timings) -> u8 {
    let probe_url = env::var(PROBE_URL_VARIABLE).unwrap_or_else(|_| DEFAULT_PROBE_URL.to_string());
    let tunnel = Tunnel::new(
        cli,
        Arc::new(HttpBlockedSiteProbe::new(probe_url, timings.probe_timeout)),
        timings,
    );
    let finder = Finder::new(
        paths.clone(),
        catalog::bundled(),
        tunnel,
        Arc::new(TcpReachabilityProbe::new(REACH_PORT, timings.reach_timeout)),
        AttemptLog::new(paths, Box::new(SystemClock)),
        Box::new(|event: EngineEvent| print_line(&event.text())),
    );
    let stop = StopSignal::new();
    listen_for_stop(&stop);
    let outcome = match mode {
        Mode::Watch => finder.watch(&stop),
        Mode::Survey => finder.survey(&stop),
        _ => finder.find(&stop),
    };
    exit_code_of(outcome)
}

/// Prints `--locations`, `--reach` or `--rank`, which read files and probe but never touch the tunnel.
fn run_listing(mode: Mode, paths: &Paths, timings: Timings) -> u8 {
    let configuration = Configuration::load(&paths.configuration_file());
    let datacenters = configuration.select(&catalog::load(&paths.windscribe_log_directory));
    match mode {
        Mode::Locations => {
            for datacenter in &datacenters {
                print_line(&format!(
                    "{}: {}",
                    datacenter.nickname,
                    datacenter.addresses.join(" ")
                ));
            }
        }
        Mode::Reach => {
            let probe = Arc::new(TcpReachabilityProbe::new(REACH_PORT, timings.reach_timeout));
            if let Ok(reachable) = reachability::reachable(&datacenters, probe, &StopSignal::new())
            {
                for datacenter in reachable {
                    print_line(&datacenter.nickname);
                }
            }
        }
        _ => {
            print_line("protocols (from Settings):");
            for protocol in &configuration.protocol_order {
                print_line(protocol);
            }
            print_line("locations (by recent success):");
            let names: Vec<String> = datacenters
                .into_iter()
                .map(|datacenter| datacenter.nickname)
                .collect();
            let recent = AttemptLog::new(paths.clone(), Box::new(SystemClock))
                .read_recent(RANK_WINDOW_LINES);
            for name in ranking::rank(&names, &recent) {
                print_line(&name);
            }
        }
    }
    SUCCESS_EXIT_CODE
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let mode = match Mode::parse(&arguments) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(USAGE_EXIT_CODE);
        }
    };
    let paths = Paths::for_current_user();
    // The catalog needs no CLI, so Settings can list locations before Windscribe is installed.
    if mode == Mode::Catalog {
        print_line(&catalog_output::render(&catalog::load(
            &paths.windscribe_log_directory,
        )));
        return ExitCode::from(SUCCESS_EXIT_CODE);
    }
    let cli = ProcessWindscribeCli::new(paths.windscribe_cli.clone());
    if !cli.is_installed() {
        print_line(&EngineEvent::CliMissing(paths.windscribe_cli.clone()).text());
        return ExitCode::from(CLI_MISSING_EXIT_CODE);
    }
    let timings = Timings::from_environment();
    let code = match mode {
        Mode::Status => {
            print_line(&TunnelStatus::parse(&cli.read_status()).status_line());
            SUCCESS_EXIT_CODE
        }
        Mode::Find | Mode::Watch | Mode::Survey => run_search(mode, paths, Arc::new(cli), timings),
        Mode::Reach | Mode::Rank | Mode::Locations => run_listing(mode, &paths, timings),
        // Already printed above, before the CLI check.
        Mode::Catalog => SUCCESS_EXIT_CODE,
    };
    ExitCode::from(code)
}
