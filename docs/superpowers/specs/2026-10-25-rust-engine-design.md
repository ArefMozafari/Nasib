# One engine in Rust for every platform

Status: design, not yet built.

## Why

Nasib has two engines that do the same job. macOS runs `macOS/Engine/ws-find.sh`, a zsh script. Windows
runs `Windows/src/Nasib.Engine`, a C# port of that script with its own test suite. A Linux version is
next, and a third engine would mean a third copy of every rule and a third place for the copies to drift.
They have drifted already: Windows learned to treat a CLI error as a torn-down tunnel and to start from a
bundled datacenter list, and macOS never got either.

The engine is rewritten once, in Rust, as a small command-line program that every app runs. It is done
now, before Linux, so Linux starts on the shared engine instead of adding a platform to port later.

Success looks like this: both apps run the same `nasib-engine` binary, behave the way they do today
on the user's real network, and the zsh script and the C# engine are gone from the repository.

## Goals

- One engine, `nasib-engine`, built from one Rust crate, used by the macOS app and the Windows app.
- The engine builds and its tests pass on Linux, so the Linux app (a later unit) only adds a UI.
- Behavior stays what users have today: the same search order, timings, files and output lines.
- Where the two current engines differ, the engine takes the better behavior and this document names it.

## Non-goals

- The Linux app itself. It is a separate unit that builds on this one.
- The network connectivity check (tracked in #12). The engine should leave an obvious place for it, nothing more.
- Release CI (tracked in #13).
- Any change to the UI of either app beyond what the switch forces.
- Intel Macs, ARM Windows, or ARM Linux builds.

## Anti-goals

These would make the rewrite a failure even if every test passes.

- A search that behaves differently on a filtered network: a different order, shorter or longer waits,
  or a connection counted as working without the blocked-site check.
- Losing anyone's history or settings: `log.csv`, `last_good`, or the cities and protocols saved in Settings.
- A Stop that no longer stops promptly, or a stopped survey that leaves the user offline.
- An engine that posts its own notifications or opens windows. The apps own all UI.

## Constraints

- macOS 15 or later on Apple silicon; Windows 10 and 11 on x64; Linux on x64 (glibc).
- The engine drives `windscribe-cli` only. Windscribe's Anti-Censorship options can't be set through it, and
  the engine must not read or write Windscribe's own settings or registry values.
- The blocked-site probe must bypass any system proxy, so a tool like v2rayN can't answer in the tunnel's place.
- The Windows download stays self-contained: nothing for the user to install beyond Windscribe.
- No telemetry and no network traffic other than the reachability knocks, the blocked-site probe, and the
  app's existing update check.

## What the engine is

A single executable, `nasib-engine` (`nasib-engine.exe` on Windows), in a new top-level `Engine/` folder
holding one Cargo crate. The apps start it as a child process, read its standard output line by line, and
stop it when the user presses Stop.

### Modes

The modes are the ones `ws-find.sh` has today, with the same flags, plus `--status`.

| Invocation | What it does |
|---|---|
| `nasib-engine` | Searches once: the last pair that worked first, then every reachable datacenter by rank on each protocol in the configured order. Stops at the first pair that carries traffic. |
| `nasib-engine --watch` | Exits at once if the tunnel is up and the blocked site loads; otherwise prints `tunnel down` and searches as above. |
| `nasib-engine --survey` | Tries every pair, logs every result, never stops early, then reconnects the last pair that worked. |
| `nasib-engine --reach` | Prints the datacenters with at least one address answering on TCP 443. |
| `nasib-engine --rank` | Prints the protocol order and the datacenters ordered by recent success. |
| `nasib-engine --locations` | Prints each datacenter and its addresses. |
| `nasib-engine --status` | New. Reads `windscribe-cli status` once and prints one line, `connected: <location> / <protocol>` or `down`, without touching the tunnel. |

`--status` is new so the three-second status read moves into the engine too. Today each app parses
`windscribe-cli status` itself; afterwards neither does, and the Linux app won't need to.

`--status` passes the CLI's own text through, as both apps show it today. The location is what follows
`Connect state: Connected: ` (for example `Atlanta - Magic City`), and the protocol is what follows
`Protocol: ` up to the port (for example `IKEv2`). It does not map these to the bare nickname or the Settings
protocol names. A search's `connected:` line keeps using the bare nickname and the Settings name, as it does
today, and both apps already show either form. Anything other than a Connected state with both fields
present prints `down`. *(My call: no UI change, and no table of CLI protocol names to guess at.)*

### Windscribe CLI missing

When the CLI isn't at its path, every mode prints `windscribe-cli missing: <path>` and exits with a
non-zero code before doing anything else. The apps show their existing "CLI missing" state from that line,
so the CLI path lives only in the engine. *(My call.)*

### Output contract

The apps understand these lines, which are exactly what `ws-find.sh` prints today. The engine prints them
unchanged; anything else is free-form detail that the apps show as is.

- `trying: <nickname> / <protocol>`
- `connected: <nickname> / <protocol>`
- `failed: <nickname> / <protocol>`
- `tunnel down`
- `nothing worked`
- `search stopped`
- `survey stopped`
- `survey finished`
- `search already running`
- `down` (from `--status` only)
- `windscribe-cli missing: <path>` (new)

The contract is written down in `Engine/README.md` and covered by the engine's tests, because three apps
will depend on it.

### Search behavior

The behavior is the current one, step by step:

1. Read the configuration (cities and protocol order) once at the start of a run.
2. Build the datacenter list (see Datacenters below), filtered to the configured cities.
3. Outside a survey, try the last pair that worked first.
4. Knock on every address on TCP 443 at once, 2-second timeout. Keep the datacenters with any answer. If none
   answers, print `no address answered; trying all locations` and keep them all.
5. Rank datacenters by success rate over the last 300 lines of `log.csv`. Ties, including never-tried
   datacenters, keep their list order.
6. Print `protocol order (from Settings): ...` and `location order (by recent success): ...`.
7. For each protocol in order, for each datacenter in rank order, make one attempt.

One attempt:

1. Print `trying: ...`.
2. If the tunnel is not down, disconnect and wait.
3. Issue a non-blocking connect.
4. Poll status every second for up to 10 seconds until it reports Connected.
5. Wait 2 more seconds, because the CLI can report Connected and drop moments later.
6. The pair works only when status says Connected and
   `https://www.youtube.com/generate_204` answers 204 within 8 seconds, with no proxy.
7. On success, append an `ok` line to `log.csv`, write `last_good`, and print `connected: ...`.
8. On failure, append a `fail` line, print `failed: ...`, and disconnect and wait.

Disconnect and wait: issue a disconnect, then poll every second for up to 15 seconds until status reports
Disconnected or Error. Error counts as down because the CLI keeps reporting it after a failed connect. This is
the Windows engine's behavior; `ws-find.sh` waited the full 15 seconds in that case.

### Stopping

The apps need a stop that works the same on every platform, and Windows has no Ctrl-C for a child process
without a console. The engine therefore stops on any of these: SIGINT or SIGTERM where they exist, the line
`stop` on standard input, or standard input closing. The macOS app keeps sending SIGINT; the Windows app
writes `stop`.

On stop:

- **A find or watch:** abandon the attempt in progress without logging it, print `search stopped`,
  disconnect and wait, then exit with code 130. It does not reconnect, because that would usually retry the
  server whose failure started the search.
- **A survey:** abandon the attempt, print `survey stopped`, release the run lock, disconnect and wait, then
  reconnect the last pair that worked, logging that attempt as usual, and exit with code 130.

"Promptly" means every wait in the engine, including the blocked-site request and the reachability knocks,
gives up as soon as a stop arrives, not when its own timeout ends. The current zsh test
`should_stop_a_find_promptly_when_interrupted` sets the bar.

The clean-up after a stop isn't itself stoppable. A second stop, or standard input closing, during the
disconnect or the survey's reconnect is ignored, so the clean-up always finishes. That keeps a double-press
or a quitting app from leaving the user offline after a survey.

The app quitting or crashing closes standard input, so it counts as a stop. The engine finishes the clean-up
on its own and then exits, so a run never outlives its app by more than the clean-up. Today the Windows
engine dies with the app, leaving the tunnel as it was, and the zsh script keeps running to the end.
*(My call: this sits between the two current behaviors.)*

The apps must give the engine a real pipe for standard input. A macOS app's own standard input is
`/dev/null`, which the engine would read as closed and stop at once.

### One run at a time

Only one search, watch or survey runs at once across all processes, as the zsh lock directory and the C#
semaphore ensure today. The engine takes an exclusive OS file lock on `lock` in the state folder; if it
can't, it prints `search already running` and exits 0. An OS lock is released when the process dies, so a
crash can't leave a stale lock. The zsh script's lock directory could. `--status`, `--reach`, `--rank` and
`--locations` don't take the lock.

On macOS the lock file is `~/.ws-find/lock`, which is where the zsh script made its lock directory. If that
path is an empty directory left over from the script, the engine removes it before taking the lock.

### Datacenters

The list is a bundled seed list (today `Windows/src/Nasib.Engine/SeedDatacenters.json`, moved into the
crate and embedded in the binary), plus every datacenter in Windscribe's client log, merged by nickname. The
seed order comes first, the log adds addresses to known nicknames, and nicknames only the log knows are
added at the end. Lines are matched both as JSON objects with a `msg` field and as plain text, which covers
both log formats. Both `client.1.log` and `client.log` are read, and missing or locked files are skipped.

This is the Windows engine's behavior. On macOS it replaces the script's hard failure when the log has no
datacenters.

### Files

The state folder stays where each platform keeps it today, so nobody's history moves:

| Platform | State folder | Windscribe log folder | CLI |
|---|---|---|---|
| macOS | `~/.ws-find` | `~/Library/Application Support/Windscribe/Windscribe2` | `/usr/local/bin/windscribe-cli` |
| Windows | `%LOCALAPPDATA%\Nasib` | `%LOCALAPPDATA%\Windscribe\Windscribe2` | `%ProgramFiles%\Windscribe\windscribe-cli.exe` |
| Linux | `$XDG_DATA_HOME/nasib`, defaulting to `~/.local/share/nasib` | to be confirmed, expected `~/.local/share/Windscribe/Windscribe2` | to be confirmed, expected `/usr/bin/windscribe-cli` |

The Linux paths are unverified until the Linux unit runs Windscribe on Linux; this unit only needs them to
exist as defaults that tests can override.

`log.csv` and `last_good` keep their exact formats: `YYYY-MM-DDTHH:MM:SS,<nickname>,<protocol>,ok|fail` in
local time, and `<nickname>|<protocol>`.

The configuration becomes one format on every platform: `config.json`, as Windows writes it today
(`{ "cities": [...], "protocolOrder": [...] }`). Unknown protocols are dropped, duplicates removed, and a
missing or unreadable file means the defaults. An empty city list means every city.

**Migrating macOS settings:** the macOS app writes the zsh-syntax `~/.ws-find/config` today. On its first
launch with the new engine, if `config.json` is missing and `config` exists, the app parses `config` with
its existing parser, writes `config.json`, and deletes `config`. The engine reads only `config.json`.

For tests, environment variables override the state folder, the log folder, the CLI path and the probe URL,
and one more scales every timing down. The apps never set them.

### Notifications

The engine never posts a notification. The `NASIB_NOTIFY` switch and the script's `osascript` call go away,
since the apps already post their own from the output lines.

## How each app uses it

### macOS

- `build.sh` builds the crate for `aarch64-apple-darwin` and copies `nasib-engine` into
  `Nasib.app/Contents/MacOS/`, which signs with the app. `ws-find.sh` is no longer bundled.
- `Engine.swift` runs `nasib-engine` directly instead of `/bin/zsh ws-find.sh`, with the same arguments, and
  still interrupts it for Stop.
- The three-second status read runs `nasib-engine --status` instead of `windscribe-cli status`.
  `EngineOutputParser.parseStatus` is replaced by parsing that single line.
- `EngineConfig` writes `config.json` and runs the one-time migration above.
- `build.sh release` runs `cargo test` alongside `swift test`.

### Windows

- `build.ps1` builds the crate for `x86_64-pc-windows-msvc`. The release zip holds `Nasib.exe` and
  `nasib-engine.exe` side by side, and the app looks for the engine next to itself.
- `EngineController` starts the engine as a child process with no console window, reads its output lines,
  and writes `stop` to its standard input for Stop. It turns lines into events with a C# port of the
  macOS `EngineOutputParser`, so the Windows UI keeps reacting exactly as it does now.
- The quiet once-a-minute check runs `--watch`, which exits at once when the tunnel is healthy. The
  status read every three seconds runs `--status`.
- `Windows/src/Nasib.Engine` and `Windows/tests/Nasib.Engine.Tests` are deleted. The settings code
  (`EngineConfiguration`, `StatePaths`) moves into the app project, since only the Settings tab needs it.
- Building now needs the Rust toolchain on the Windows build machine as well as the .NET 9 SDK.

### Linux

Out of scope here. The engine builds for `x86_64-unknown-linux-gnu` and its tests pass in a Linux container.

## Tests

- The Rust tests cover everything both current suites cover:
  - the C# engine tests: log parsing, catalog merge, ranking, attempt log, configuration, status parsing,
    reachability filter, tunnel controller, server finder, timings and events;
  - the zsh engine tests: one attempt when searching while connected or disconnected, the `failed:` line,
    a prompt stop, and protocols tried in the configured order.
- Unit tests run against fakes for the CLI, the probes and the clock, so they never touch the real VPN or
  the network and don't really sleep.
- End-to-end tests run the built binary against a fake `windscribe-cli` in a temporary state folder, the
  way the zsh tests do today.
- The rules for timing and one-shot guards get explicit before-and-after tests:
  - the 10-second connect cutoff;
  - the 15-second disconnect cutoff;
  - the 2-second settle delay;
  - Error ending the disconnect wait;
  - the run lock;
  - stop during each step.
- The apps keep their own tests. The macOS parser tests gain `--status` lines; the Windows app gains tests
  for its new parser and for the stop line.

## Order of work

1. **The engine plus the macOS switch, in one pull request.** It merges on its own. `ws-find.sh` and its zsh
   tests are deleted in it, and it is verified on the real VPN on this Mac before review.
2. **The Windows switch, in a second pull request.** The C# engine is deleted in it, and it is verified on the
   Windows PC.
3. **A release (0.4.0) only after both,** so the two downloads never run different engines.

## Left to the builder

- Crate layout, module names and dependencies, within the repo's rules: full-word names, no magic values,
  one test file per source file.
- Whether to use threads or an async runtime, as long as every wait stops promptly.
- How the fake CLI for end-to-end tests is built.
- Exit codes other than 0 for success and 130 for a stop.
