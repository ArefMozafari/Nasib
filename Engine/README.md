# nasib-engine

The engine every Nasib app runs. It finds a Windscribe datacenter and protocol that carries traffic right
now, connects to it, and keeps it connected. It drives the Windscribe app's own `windscribe-cli` and never
touches Windscribe's settings.

The apps start it as a child process, read its standard output line by line, and stop it when the user
presses Stop. It can also be run by hand.

## Modes

| Invocation | What it does |
|---|---|
| `nasib-engine` | Searches once: the last pair that worked first, then every reachable datacenter by rank on each protocol in the configured order. Stops at the first pair that carries traffic. |
| `nasib-engine --watch` | Exits at once if the tunnel is up and the blocked site loads; otherwise prints `tunnel down` and searches. |
| `nasib-engine --survey` | Tries every pair, logs every result, never stops early, then reconnects the last pair that worked. |
| `nasib-engine --reach` | Prints the datacenters with at least one address answering on TCP 443. |
| `nasib-engine --rank` | Prints the protocol order and the datacenters ordered by recent success. |
| `nasib-engine --locations` | Prints each datacenter and its addresses. |
| `nasib-engine --status` | Reads `windscribe-cli status` once and prints one line, without touching the tunnel. |

A pair works only when the CLI reports Connected and `https://www.youtube.com/generate_204` answers 204
through the tunnel within 8 seconds, with any system proxy bypassed. Each attempt gets 10 seconds to
connect and 2 more to settle.

## Output contract

The apps act on these lines, so they don't change without changing every app. Anything else the engine
prints is detail that the apps show as it is.

| Line | Meaning |
|---|---|
| `trying: <nickname> / <protocol>` | An attempt started. |
| `connected: <nickname> / <protocol>` | A blocked site loaded through the tunnel. |
| `failed: <nickname> / <protocol>` | An attempt failed. Normal while searching. |
| `tunnel down` | `--watch` found the tunnel not working and is searching. |
| `nothing worked` | Every pair failed. |
| `search stopped` | A find or watch was stopped; the tunnel is left down. |
| `survey stopped` | A survey was stopped; the last pair that worked is reconnected. |
| `survey finished` | A survey tried every pair. |
| `search already running` | Another run holds the lock; this one did nothing. |
| `connected: <location> / <protocol>` | From `--status`: the CLI's own location and protocol, such as `Atlanta - Magic City / IKEv2`. |
| `down` | From `--status`: anything other than a fully connected tunnel. |
| `windscribe-cli missing: <path>` | The Windscribe CLI isn't installed where expected. Every mode checks this first. |

## Stopping

A find, watch or survey stops on any of these:

- Ctrl-C or SIGTERM;
- the line `stop` on standard input;
- standard input closing, which is what happens when the app that started it quits.

So give it a real pipe or a terminal for standard input. Started with `</dev/null`, it stops at once.

After a stop, the clean-up always finishes:
- a find, or a watch that has already printed `tunnel down`, disconnects;
- a survey reconnects the last pair that worked;
- a watch stopped during its health check leaves the tunnel as it is.

Further stops during the clean-up are ignored, and the clean-up still runs when the app has gone and
nothing reads the output any more.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | A pair works, a watch found the tunnel healthy, a listing was printed, or another run was already going |
| 1 | Nothing worked, or no datacenter matched the configured cities |
| 2 | `windscribe-cli` is missing |
| 64 | Unknown arguments |
| 130 | Stopped |

## Files

| Platform | State folder | Windscribe log folder | CLI |
|---|---|---|---|
| macOS | `~/.ws-find` | `~/Library/Application Support/Windscribe/Windscribe2` | `/usr/local/bin/windscribe-cli` |
| Windows | `%LOCALAPPDATA%\Nasib` | `%LOCALAPPDATA%\Windscribe\Windscribe2` | `%ProgramFiles%\Windscribe\windscribe-cli.exe` |
| Linux | `$XDG_DATA_HOME/nasib` (default `~/.local/share/nasib`) | `$XDG_DATA_HOME/Windscribe/Windscribe2` | `/usr/bin/windscribe-cli` |

The Linux locations are the expected ones, not yet confirmed against a Linux install of Windscribe.

The state folder holds:

| File | What's in it |
|---|---|
| `log.csv` | Every attempt: `YYYY-MM-DDTHH:MM:SS,<nickname>,<protocol>,ok` or `fail`, in local time |
| `last_good` | The last pair that worked, as `<nickname>\|<protocol>` |
| `config.json` | `{ "cities": [...], "protocolOrder": [...] }`, written by the app's Settings. An empty city list means every city. |
| `lock` | Held while a find, watch or survey runs |

Datacenters come from a bundled list (`src/seed_datacenters.json`) merged with every datacenter in
Windscribe's client log, so new or moved servers show up without an update.

## Building and testing

```bash
cd Engine
cargo test
cargo build --release
```

- The tests run against fakes for the CLI and the probes, so they never touch the real VPN.
- On macOS and Linux, `tests/main.rs` also runs the built binary against `tests/support/fake-windscribe-cli`.
- These environment variables are for tests only, and the apps never set them:
  - `NASIB_STATE_DIR`
  - `NASIB_WINDSCRIBE_LOG_DIR`
  - `NASIB_WINDSCRIBE_CLI`
  - `NASIB_PROBE_URL`
  - `NASIB_TIME_SCALE`, which divides every timing.
