# Rust Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `macOS/Engine/ws-find.sh` and `Windows/src/Nasib.Engine` with one Rust program, `nasib-engine`, that both apps run as a child process.

**Architecture:** One Cargo crate in `Engine/`, built as a library (all logic, behind traits for the CLI, the probes, the clock and the waits) plus a thin binary (argument parsing, real implementations, stop listeners). The apps start the binary, read its output lines, and stop it with SIGINT (macOS) or a `stop` line on standard input (Windows).

**Tech Stack:** Rust 2024 edition (toolchain 1.99 on this Mac). Dependencies:
- `serde` and `serde_json` for `config.json`, the seed list and the client log;
- `ureq` 3 with rustls, proxy off, for the blocked-site probe;
- `ctrlc` with the `termination` feature, for SIGINT and SIGTERM;
- `chrono` for local timestamps.

Swift 6 / SwiftUI for the macOS app; C# / .NET 9 / WPF for the Windows app.

**Spec:** `docs/superpowers/specs/2026-10-25-rust-engine-design.md`

## Global Constraints

- Output lines exactly as listed under "Output contract" in the spec; nothing else is parsed by the apps.
- Timings: probe 8 s, connect 10 s, settle 2 s, disconnect 15 s, status poll 1 s, reach 2 s on TCP 443; rank window 300 lines.
- Probe URL `https://www.youtube.com/generate_204`, success only on HTTP 204, no proxy, a fresh connection per probe.
- Default protocols in order: `wireguard ikev2 tcp stealth wstunnel udp`. Default cities: Istanbul, Dubai, Sofia, Stockholm, Amsterdam, Frankfurt, Zurich, Toronto, Montreal, Winnipeg, New York, Chicago, Atlanta, Bratislava, Helsinki, Brussels.
- `log.csv` line: `YYYY-MM-DDTHH:MM:SS,<nickname>,<protocol>,ok|fail`, local time. `last_good`: `<nickname>|<protocol>\n`.
- Exit codes: 0 success, healthy watch, or `search already running`; 1 nothing worked or no datacenters; 2 `windscribe-cli missing`; 130 stopped.
- Test overrides, never set by the apps: `NASIB_STATE_DIR`, `NASIB_WINDSCRIBE_LOG_DIR`, `NASIB_WINDSCRIBE_CLI`, `NASIB_PROBE_URL`, `NASIB_TIME_SCALE` (a divisor applied to every timing).
- Repo rules: full-word identifiers (no `i`, `ctx`, `dc`), comments above the code as full sentences, helpers defined above their callers, no magic values, one test file per source file, test names `should_...`.
- Commits: small, by concern, house style from `git log` (`feat:`, `build:`, `docs:`, `refactor:`, `test:`), no attribution. Nothing is committed or pushed without the user's go-ahead.

## Review Focus

1. **A Windscribe log with a datacenter name that has no ` - `, or a nickname containing ` / `.** The expected behavior is that the city is the whole name, and that the apps split `trying:` lines on the last ` / `. Pinned in Task 3 and Task 12.
2. **`config.json` that is valid JSON but has `protocolOrder: []`.** The expected behavior is that the search tries no protocols and prints `nothing worked`, as today, without crashing. Pinned in Task 2.
3. **A stop that arrives while the blocked-site request is waiting on a dead network.** The expected behavior is that it ends within one status poll, not after 8 seconds. Pinned in Task 7.
4. **A leftover `~/.ws-find/lock` directory from the zsh script.** The expected behavior is that it is removed and the first run proceeds. If it isn't empty, the run reports `search already running` rather than deleting anything. Pinned in Task 6.
5. **`log.csv` with malformed lines or a trailing partial line from a crash.** The expected behavior is that those lines are skipped by ranking and the next append still starts on its own line. Pinned in Task 4.

---

## Part A: the engine and the macOS switch (one pull request)

### Task 1: Crate skeleton, timings and the stop signal

**Files:**
- Create: `Engine/Cargo.toml`, `Engine/src/lib.rs`, `Engine/src/main.rs`, `Engine/src/timings.rs`, `Engine/src/stop.rs`, `Engine/.gitignore` (`/target`)
- Test: `Engine/tests/timings.rs`, `Engine/tests/stop.rs`

**Interfaces:**
- Produces:
  - `pub struct Timings { pub probe_timeout: Duration, pub connect_timeout: Duration, pub settle_delay: Duration, pub disconnect_timeout: Duration, pub status_poll_interval: Duration, pub reach_timeout: Duration }`;
  - `Timings::default()`;
  - `Timings::scaled(divisor: f64) -> Timings`;
  - `Timings::from_environment() -> Timings`, which reads `NASIB_TIME_SCALE`.
  - `pub struct StopSignal` (Clone; `Arc<(Mutex<StopState>, Condvar)>`) with:
    - `new()`;
    - `request()`;
    - `is_requested() -> bool`;
    - `wait(&self, duration: Duration) -> Result<(), Stopped>`, which returns early with `Err(Stopped)` on request;
    - `shield(&self) -> StopSignal`, a signal that ignores every request, used for clean-up.
  - `pub struct Stopped;`

- [ ] **Step 1: Write the failing tests:**
  - `should_match_ws_find_defaults`: the six values from Global Constraints.
  - `should_divide_every_timing_by_the_scale`.
  - `should_return_stopped_when_requested_during_wait`: request from another thread after 20 ms, so that `wait(10 s)` returns `Err` in under 200 ms.
  - `should_wait_the_full_duration_without_a_request`.
  - `should_ignore_requests_on_a_shielded_signal`.
- [ ] **Step 2:** `cargo test --manifest-path Engine/Cargo.toml` fails, because the types don't exist.
- [ ] **Step 3:** Implement. `main.rs` prints nothing yet and exits 0.
- [ ] **Step 4:** `cargo test` passes.
- [ ] **Step 5:** Commit `build: add the Rust engine crate with timings and a stop signal`.

### Task 2: Configuration and state paths

**Files:**
- Create: `Engine/src/configuration.rs`, `Engine/src/paths.rs`
- Test: `Engine/tests/configuration.rs`, `Engine/tests/paths.rs`
- Reference for behavior: `Windows/src/Nasib.Engine/EngineConfiguration.cs` and `Windows/src/Nasib.Engine/StatePaths.cs`

**Interfaces:**
- Produces:
  - `pub struct Configuration { pub cities: Vec<String>, pub protocol_order: Vec<String> }`, with `Configuration::default()`, `Configuration::load(path: &Path) -> Configuration` and `includes_city(&self, city: &str) -> bool`, which is case-insensitive and true for any city when `cities` is empty.
  - `pub const ALL_PROTOCOLS: [&str; 6]`.
  - `pub struct Paths { pub state_directory: PathBuf, pub windscribe_log_directory: PathBuf, pub windscribe_cli: PathBuf }`, with:
    - `log_file()`, `last_good_file()`, `configuration_file()` (named `config.json`) and `lock_file()`;
    - `Paths::for_current_user() -> Paths`, using the per-platform defaults from the spec's Files table plus the `NASIB_*` overrides.
- [ ] **Step 1:** Port every case from `Windows/tests/Nasib.Engine.Tests/EngineConfigurationTests.cs` and `StatePathsTests.cs` as `should_...` tests. Add these:
  - `should_keep_an_empty_protocol_order`, from Review Focus 2;
  - `should_drop_unknown_and_duplicate_protocols`;
  - `should_use_dot_ws_find_on_macos`, a `#[cfg(target_os = "macos")]` test;
  - `should_prefer_environment_overrides`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement. Paths go through `std::env::home_dir`, `LOCALAPPDATA`, `ProgramFiles` and `XDG_DATA_HOME`.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: read the engine configuration and state paths`.

### Task 3: Datacenters, the client log and the seed list

**Files:**
- Create: `Engine/src/datacenter.rs`, `Engine/src/client_log.rs`, `Engine/src/catalog.rs`
- Move: `Windows/src/Nasib.Engine/SeedDatacenters.json` → `Engine/src/seed_datacenters.json` (git mv in Task 15; copy it now), embedded with `include_str!`
- Test: `Engine/tests/datacenter.rs`, `Engine/tests/client_log.rs`, `Engine/tests/catalog.rs`

**Interfaces:**
- Produces:
  - `pub struct Datacenter { pub city: String, pub nickname: String, pub addresses: Vec<IpAddr> }`, with `Datacenter::from_location_name(name: &str, addresses: Vec<IpAddr>)`, which splits on the first ` - `, and `with_addresses(&self, more: &[IpAddr]) -> Datacenter`, which merges without duplicates in first-seen order.
  - `client_log::parse_line(line: &str) -> Option<Datacenter>`.
  - `client_log::read_directory(directory: &Path) -> Vec<Datacenter>`, which reads `client.1.log` then `client.log`.
  - `catalog::load(configuration: &Configuration, log_directory: &Path) -> Vec<Datacenter>`, which is the seed merged with the log and filtered by city.
- [ ] **Step 1:** Port every case from the C# tests `DatacenterTests`, `ClientLogParserTests`, `DatacenterCatalogTests` and `SeedDatacentersTests`. Add these:
  - `should_parse_the_macos_plain_text_line`, with the escaped-quote form `MutableLocationInfo created:  \"Dubai - Khalifa\" \"135.136.7.178; \"` that `ws-find.sh` greps for;
  - `should_treat_a_name_without_separator_as_the_city`, from Review Focus 1.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement. One regex isn't worth a dependency, so match with `str` methods. Handle both the escaped and the unescaped quote forms.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: load datacenters from the seed list and the Windscribe log`.

### Task 4: Attempt log and ranking

**Files:**
- Create: `Engine/src/attempt_log.rs`, `Engine/src/ranking.rs`
- Test: `Engine/tests/attempt_log.rs`, `Engine/tests/ranking.rs`

**Interfaces:**
- Produces:
  - `pub trait Clock { fn local_timestamp(&self) -> String; }` and `SystemClock`.
  - `pub struct AttemptLog { paths, clock }` with `record(nickname, protocol, succeeded)`, `read_recent(count: usize) -> Vec<AttemptRecord>`, `read_last_good() -> Option<ServerPair>` and `write_last_good(&ServerPair)`.
  - `ranking::rank(names: &[String], recent: &[AttemptRecord]) -> Vec<String>`, which sorts by success rate descending, and keeps the input order on ties.
- [ ] **Step 1:** Port the C# `AttemptLogTests` and `SuccessRankerTests`. Add these, from Review Focus 5:
  - `should_skip_malformed_lines_when_ranking`;
  - `should_start_a_new_line_after_a_partial_last_line`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement. Before appending, `record` checks whether the file ends without `\n` and writes one first.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: record attempts and rank datacenters by recent success`.

### Task 5: Tunnel status and the Windscribe CLI

**Files:**
- Create: `Engine/src/tunnel_status.rs`, `Engine/src/windscribe_cli.rs`
- Test: `Engine/tests/tunnel_status.rs`, `Engine/tests/windscribe_cli.rs`

**Interfaces:**
- Produces:
  - `pub enum ConnectState { Unknown, Disconnected, Connecting, Connected, Disconnecting, Error }`.
  - `pub struct TunnelStatus { pub state, pub location: Option<String>, pub protocol: Option<String> }`, with `parse(output: &str)`, `is_connected()`, `is_down()` (true for Disconnected or Error) and `status_line(&self) -> String`, which returns `connected: <location> / <protocol>` or `down`.
  - `pub trait WindscribeCli: Send + Sync { fn read_status(&self) -> String; fn connect(&self, nickname: &str, protocol: &str); fn disconnect(&self); }`.
  - `ProcessWindscribeCli::new(path: PathBuf)` and `is_installed()`. The connect arguments are `connect -n <nickname> <protocol>`. Each call has a 60-second guard timeout. On Windows the CLI runs with `CREATE_NO_WINDOW`.
- [ ] **Step 1:** Port the C# `TunnelStatusTests` and `WindscribeCliTests`, which check the argument lists. Port the macOS `EngineOutputParserTests` `parseStatus` cases as `status_line` tests. Add `should_print_down_when_connected_without_a_protocol`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: read tunnel status through windscribe-cli`.

### Task 6: Run lock

**Files:**
- Create: `Engine/src/run_lock.rs`
- Test: `Engine/tests/run_lock.rs`

**Interfaces:**
- Produces: `RunLock::acquire(path: &Path) -> Option<RunLock>`, which is `None` when another process holds it and releases on drop. It uses `std::fs::File::try_lock`.
- [ ] **Step 1:** Write these tests:
  - `should_refuse_a_second_lock_while_held`;
  - `should_grant_the_lock_after_release`;
  - `should_remove_an_empty_leftover_lock_directory`, from Review Focus 4;
  - `should_refuse_when_the_lock_path_is_a_non_empty_directory`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: allow one engine run at a time with a file lock`.

### Task 7: Probes

**Files:**
- Create: `Engine/src/blocked_site.rs`, `Engine/src/reachability.rs`
- Test: `Engine/tests/blocked_site.rs`, `Engine/tests/reachability.rs`

**Interfaces:**
- Produces:
  - `pub trait BlockedSiteProbe: Send + Sync { fn loads(&self, stop: &StopSignal) -> Result<bool, Stopped>; }`.
  - `HttpBlockedSiteProbe::new(url: String, timeout: Duration)`.
  - `pub trait ReachabilityProbe: Send + Sync { fn answers(&self, address: IpAddr, stop: &StopSignal) -> Result<bool, Stopped>; }`.
  - `TcpReachabilityProbe::new(port: u16 /* 443 */, timeout)`.
  - `reachability::reachable(datacenters: &[Datacenter], probe: &dyn ReachabilityProbe, stop: &StopSignal) -> Result<Vec<Datacenter>, Stopped>`. It knocks on every address concurrently with scoped threads and keeps the input order.
- [ ] **Step 1:** Port the C# `HttpBlockedSiteProbeTests`, `TcpReachabilityProbeTests` and `ReachabilityFilterTests`. They use a local `TcpListener` and a tiny local HTTP responder that answers 204 or 200. Add these:
  - `should_return_stopped_while_the_request_hangs`, from Review Focus 3: a listener that accepts and never answers, a stop after 50 ms, and a return in under 500 ms;
  - `should_ignore_a_system_proxy`: set `HTTPS_PROXY` to a dead port, and the probe still reaches the local responder.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement. The blocked request runs on a worker thread, and the caller waits on a channel in `status_poll_interval` slices, checking `stop`. A stopped request's thread is abandoned.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: probe reachability and the blocked site, stoppable`.

### Task 8: Tunnel controller

**Files:**
- Create: `Engine/src/tunnel.rs`
- Test: `Engine/tests/tunnel.rs`, with fakes in `Engine/tests/support/mod.rs` (`FakeCli` scripted with a status sequence, `FakeBlockedSiteProbe`, `FakeReachabilityProbe` and `FixedClock`)

**Interfaces:**
- Consumes: Tasks 1, 5 and 7.
- Produces: `Tunnel { cli, probe, timings }` with:
  - `read_status() -> TunnelStatus`;
  - `disconnect_and_wait(stop) -> Result<bool, Stopped>`;
  - `connect(nickname, protocol, stop) -> Result<(), Stopped>`;
  - `works(stop) -> Result<bool, Stopped>`.

  The waits go through `stop.wait`, so tests use `Timings::scaled(1000.0)` and never really sleep.
- [ ] **Step 1:** Port the C# `TunnelControllerTests`. Add explicit before-and-after tests for each cutoff:
  - `should_stop_polling_connect_after_ten_polls`;
  - `should_end_disconnect_wait_on_error`;
  - `should_give_up_disconnect_after_fifteen_polls`;
  - `should_settle_before_probing`, checked by the call order on `FakeCli`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: connect, disconnect and check the tunnel`.

### Task 9: Server finder

**Files:**
- Create: `Engine/src/events.rs`, `Engine/src/finder.rs`
- Test: `Engine/tests/events.rs`, `Engine/tests/finder.rs`

**Interfaces:**
- Consumes: Tasks 2 to 8.
- Produces:
  - `pub enum EngineEvent { Trying(ServerPair), Connected(ServerPair), Failed(ServerPair), SearchStopped, SurveyStopped, SurveyFinished, NothingWorked, TunnelDown, SearchAlreadyRunning, CliMissing(PathBuf), Message(String) }`, with `text(&self) -> String`, which returns the exact contract line.
  - `Finder { paths, tunnel, reachability_probe, attempt_log, events: Box<dyn Fn(EngineEvent)> }` with `find(stop) -> Outcome`, `watch(stop) -> Outcome` and `survey(stop) -> Outcome`, where `pub enum Outcome { Succeeded, Failed, Stopped, AlreadyRunning }`.
- [ ] **Step 1:** Port the C# `ServerFinderTests` and `EngineEventTests`. Port the zsh behaviors as finder tests:
  - `should_log_one_attempt_when_searching_while_connected`;
  - `should_log_one_attempt_when_searching_while_disconnected`;
  - `should_try_protocols_in_the_configured_order`.

  Add these:
  - `should_not_log_the_attempt_a_stop_abandons`;
  - `should_finish_survey_reconnect_despite_a_second_stop`, which shields the clean-up;
  - `should_not_reconnect_after_a_stopped_find`;
  - `should_report_already_running_when_locked`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement, following the spec's step lists. The clean-up after a stop uses `stop.shield()`.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: find, watch and survey for a working server`.

### Task 10: The binary: modes, stop listeners, CLI check

**Files:**
- Modify: `Engine/src/main.rs`
- Create: `Engine/src/mode.rs`, `Engine/README.md`, which covers the modes table, the output contract, exit codes and the test overrides
- Test: `Engine/tests/mode.rs`; `Engine/tests/main.rs`, end to end and `#[cfg(unix)]`, running `env!("CARGO_BIN_EXE_nasib-engine")` against `Engine/tests/support/fake-windscribe-cli`, a POSIX `sh` script that keeps its state in a file and logs its calls

**Interfaces:**
- Produces:
  - `pub enum Mode { Find, Watch, Survey, Reach, Rank, Locations, Status }`;
  - `Mode::parse(arguments: &[String]) -> Result<Mode, String>`.
- [ ] **Step 1:** Write these tests:
  - `mode.rs`: each flag; an unknown flag is an error.
  - `main.rs`:
    - `should_print_cli_missing_and_exit_two`;
    - `should_print_status_line`;
    - `should_stop_a_find_promptly_on_sigint`, which ports the zsh test with the same bar;
    - `should_stop_on_a_stop_line`;
    - `should_stop_when_standard_input_closes`;
    - `should_exit_zero_on_a_healthy_watch`.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement:
  - Check `windscribe_cli` first.
  - Install a `ctrlc` handler that calls `stop.request()`.
  - Spawn a thread that reads standard input lines and calls `stop.request()` on `stop` or EOF.
  - Print each event with a flush after every line.
  - Map `Outcome` to the exit codes.
- [ ] **Step 4:** `cargo test` passes in full. `cargo clippy -- -D warnings` is clean.
- [ ] **Step 5:** Commit `feat: run the engine modes from the command line`. Then commit `docs: document the engine's modes and output` for `Engine/README.md`.

### Task 11: Linux build check

**Files:** none changed. This task is verification only.

- [ ] **Step 1:** Run `docker run --rm -v "$PWD/Engine":/engine -w /engine rust:1.99 cargo test`. All tests pass.
- [ ] **Step 2:** If Docker isn't installed, say so in the report; don't install it without asking.

### Task 12: macOS app runs the Rust engine

**Files:**
- Modify:
  - `macOS/build.sh`: build `../Engine` for `aarch64-apple-darwin` and copy it to `Contents/MacOS/nasib-engine`; `release` runs `cargo test`; drop the `ws-find.sh` copy and the zsh test line.
  - `macOS/Sources/Nasib/Engine.swift`:
    - run `Bundle.main.url(forAuxiliaryExecutable: "nasib-engine")`;
    - give the process a stdin `Pipe` that stays open;
    - drop `NASIB_NOTIFY` and `/bin/zsh`;
    - read the status from `--status`;
    - `engineMissing` checks the binary.
  - `macOS/Sources/Nasib/EngineOutputParser.swift`:
    - add `case cliMissing(path: String)` with an `.error` tone;
    - replace `parseStatus(output:)` with `parseStatus(line: String) -> TunnelStatus?`.
- Test: `macOS/Tests/NasibTests/EngineOutputParserTests.swift`

- [ ] **Step 1:** Write the tests. Rewrite the `parseStatus` tests for the single line, and add these:
  - `testShouldParseConnectedStatusLine`;
  - `testShouldReturnNilForDown`;
  - `testShouldParseCliMissing`;
  - `testShouldSplitPairOnLastSeparator`, from Review Focus 1.
- [ ] **Step 2:** Run `cd macOS && swift test --filter EngineOutputParserTests`. It fails.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** The filtered test passes. Then `swift test` passes in full.
- [ ] **Step 5:** Commit `feat: run the Rust engine from the macOS app`.

### Task 13: macOS settings move to config.json

**Files:**
- Modify: `macOS/Sources/Nasib/EngineConfig.swift`. It stays `Codable` with keys `cities` and `protocolOrder`, and gets `load()`, `save()` and `migrateLegacyFileIfNeeded()`. The zsh-syntax parser is kept only for the migration.
- Modify: `macOS/Sources/Nasib/NasibApp.swift`, which calls the migration once at launch.
- Test: `macOS/Tests/NasibTests/EngineConfigTests.swift`

- [ ] **Step 1:** Write these tests:
  - `testShouldRoundTripJson`;
  - `testShouldMigrateLegacyConfigAndDeleteIt`;
  - `testShouldNotMigrateWhenJsonExists`;
  - `testShouldKeepLegacyFileWhenItCannotBeParsed`.

  The tests run against a temporary directory, so `EngineConfig` takes its directory as a parameter with the real one as the default.
- [ ] **Step 2:** Run the tests and watch them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and watch them pass.
- [ ] **Step 5:** Commit `feat: save macOS settings as config.json and migrate the old file`.

### Task 14: Verify on the real VPN

- [ ] **Step 1:** Run `./macOS/build.sh install`, then confirm the app launches and the menu shows the current server.
- [ ] **Step 2:** Back up `~/.ws-find`, then confirm the following. Each check names what proves it:
  - `config` migrated to `config.json`, with the same cities and protocols showing in Settings;
  - status reads 3 s apart in Windscribe's log;
  - Find Now connects, and `log.csv` and `last_good` are updated in the old format;
  - Stop during a find disconnects within about 2 s;
  - Stop during a survey reconnects the last good pair;
  - auto-reconnect starts within about 3 s after a manual `windscribe-cli disconnect`.
- [ ] **Step 3:** Report what was observed, with the times.

### Task 15: Remove the zsh engine and update docs

**Files:**
- Delete: `macOS/Engine/ws-find.sh`, `macOS/Tests/Engine/ws-find.test.zsh`
- Modify: `README.md`:
  - "Building from source": a new Engine section with `cargo test`; macOS needs the Rust toolchain;
  - the standalone engine line points to `nasib-engine` and its README;
  - the files table says `config.json` on both platforms.
- [ ] **Step 1:** Delete the files. `./macOS/build.sh release` still passes.
- [ ] **Step 2:** Commit `refactor: remove the zsh engine` and `docs: describe the shared Rust engine`.
- [ ] **Step 3:** Stop. Report to the user and wait for approval to push and open the pull request.

---

## Part B: the Windows switch (second pull request, after Part A merges)

This part is run through the Windows session, since only the Windows PC can build and verify it.

### Task 16: Windows app runs the Rust engine

**Files:**
- Create:
  - `Windows/src/Nasib.App/EngineProcess.cs`. It starts `nasib-engine.exe` from `AppContext.BaseDirectory` with `CreateNoWindow` and redirected input and output. It raises `LineReceived` and `Exited(int code)`, and has `Stop()`, which writes `stop\n`.
  - `Windows/src/Nasib.App/EngineOutputParser.cs`, a port of the Swift parser including `cliMissing` and `parseStatus(line)`.
- Modify:
  - `Windows/src/Nasib.App/EngineController.cs`: it uses `EngineProcess` for find, watch and survey; the status read runs `--status`; `CliMissing` comes from a `windscribe-cli missing:` line.
  - `Windows/src/Nasib.App/App.xaml.cs`: it no longer builds `ServerFinder`.
  - `Windows/src/Nasib.App/OutputLine.cs`: it builds from parsed events.
- Move into `Windows/src/Nasib.App/`: `EngineConfiguration.cs` and `StatePaths.cs`.
- Delete: `Windows/src/Nasib.Engine/`, `Windows/tests/Nasib.Engine.Tests/`, and their entries in `Windows/Nasib.sln`.
- Modify: `Windows/build.ps1`. It runs `cargo test` and `cargo build --release --target x86_64-pc-windows-msvc`, and zips `Nasib.exe` and `nasib-engine.exe`.
- Test:
  - `Windows/tests/Nasib.App.Tests/EngineOutputParserTests.cs` and `EngineProcessTests.cs`, against a stub engine;
  - existing `EngineControllerTests.cs` updated.

- [ ] **Step 1:** Write the failing tests, then run `dotnet test Windows/Nasib.sln` and see them fail.
- [ ] **Step 2:** Implement, then run `dotnet test` again and see it pass.
- [ ] **Step 3:** Run `./Windows/build.ps1` and check the zip holds both executables.
- [ ] **Step 4:** Verify on the Windows PC with the same checks as Task 14, plus that quitting the app mid-survey leaves the tunnel on the last good pair.
- [ ] **Step 5:** Update `README.md`: the Windows install now says both files stay in one folder, and Windows building needs Rust. Commit by concern, then stop for approval.

### Task 17: Release 0.4.0

Only after both pull requests merge, and only with the user's go-ahead. Bump `macOS/Packaging/Info.plist` and `Windows/Directory.Build.props`, build both zips, and publish as for 0.3.0.
