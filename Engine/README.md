# nasib-engine

The engine every Nasib app runs. It finds a Windscribe datacenter and protocol that carries traffic right
now, connects to it, and keeps it connected. It drives the Windscribe app's own `windscribe-cli` and never
touches Windscribe's settings.

The apps start it as a child process, read its standard output line by line, and stop it when the user
presses Stop. It can also be run by hand.

## Modes

| Invocation | What it does |
|---|---|
| `nasib-engine` | Searches once and stops at the first pair that carries traffic. See "How a search runs" below. |
| `nasib-engine --watch` | Exits at once if the tunnel is up and the blocked site loads; otherwise prints `tunnel down` and searches. |
| `nasib-engine --survey` | Tries every pair on every port, logs every result, never stops early, then reconnects the last pair that worked. |
| `nasib-engine --reach` | Prints the datacenters with at least one address answering on TCP 443. |
| `nasib-engine --rank` | Prints the protocol order and the datacenters ordered by recent success. |
| `nasib-engine --locations` | Prints each datacenter and its addresses. |
| `nasib-engine --status` | Reads `windscribe-cli status` once and prints one line, without touching the tunnel. |
| `nasib-engine --catalog` | Prints every datacenter, grouped by region, as one JSON line for the apps' Settings. Needs no `windscribe-cli`. |

A pair works only when the CLI reports Connected and `https://www.youtube.com/generate_204` answers 204
through the tunnel within 8 seconds, with any system proxy bypassed. Each attempt gets 10 seconds to
connect and 2 more to settle.

## How a search runs

Each stage starts with a `phase:` line.

1. **The last pair that worked**, on its port, outside a survey.
2. **Reachability:** every address of the chosen locations is knocked on TCP 443 at once. Locations where
   nothing answers are skipped, unless nothing answers at all.
3. **First pass:** for each protocol in the configured order, its preferred port, on every location in
   order. The preferred port is the one that has worked best here, or the protocol's default port, the
   first one `windscribe-cli ports` lists. The locations keep the user's order, re-sorted by success over
   the last 300 attempts when `rankBySuccess` is on.
4. **Second pass**, only when the first found nothing, or in a survey: each protocol's other ports, in
   three groups: ports that have worked here, best first; then ports never tried, in Windscribe's order;
   then ports that only failed, least tried first. A find skips it when no address answered a knock,
   since the network itself is likely down and every other port would fail too.

The ports are read once per run, before the first attempt, since the CLI refuses a second command while
one is running. A protocol whose ports can't be read connects without a port, and its attempts log the
port the status then reports.

## Output contract

The apps act on these lines, so they don't change without changing every app. Anything else the engine
prints is detail that the apps show as it is.

| Line | Meaning |
|---|---|
| `trying: <nickname> / <protocol>:<port>` | An attempt started. Without a known port, just `<protocol>`. |
| `connected: <nickname> / <protocol>:<port>` | A blocked site loaded through the tunnel. |
| `failed: <nickname> / <protocol>:<port>` | An attempt failed. Normal while searching. |
| `phase: <stage>` | A stage of the search starts: `last pair that worked`, `reachability`, or `<protocol> on port <port>`. |
| `tunnel down` | `--watch` found the tunnel not working and is searching. |
| `nothing worked` | Every pair failed. |
| `search stopped` | A find or watch was stopped; the tunnel is left down. |
| `survey stopped` | A survey was stopped; the last pair that worked is reconnected. |
| `survey finished` | A survey tried every pair. |
| `search already running` | Another run holds the lock; this one did nothing. |
| `connected: <location> / <protocol>` | From `--status`: the CLI's own location and protocol, such as `Atlanta - Magic City / IKEv2`. |
| `down` | From `--status`: anything other than a fully connected tunnel. |
| `windscribe-cli missing: <path>` | The Windscribe CLI isn't installed where expected. Every mode but `--catalog` checks this first. |
| `{"regions":[{"name":…,"datacenters":[{"city":…,"nickname":…,"logged":true}]}]}` | From `--catalog`: regions in the bundled order; datacenters only the Windscribe log knows come last, under `Other`. `logged` appears, as `true`, on every datacenter the Windscribe log names. |

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
| 1 | Nothing worked, or no datacenter matched the locations in Settings |
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
| `log.csv` | Every attempt: `YYYY-MM-DDTHH:MM:SS,<nickname>,<protocol>,ok\|fail,<port>`, in local time. Lines from before ports were recorded have no fifth column and count as the protocol's default port. |
| `last_good` | The last pair that worked, as `<nickname>\|<protocol>:<port>`. An older `<nickname>\|<protocol>` is still read. |
| `config.json` | Written by the app's Settings; see below. |
| `lock` | Held while a find, watch or survey runs |

`config.json`:

```json
{
  "locations": ["The 6", "Khalifa"],
  "rankBySuccess": true,
  "protocolOrder": ["tcp", "ikev2"],
  "cities": ["Toronto", "Dubai"]
}
```

- `locations`: the datacenters to search, by nickname, in order. An empty list means every datacenter.
- `rankBySuccess`: re-sorts the locations by recent success. It's `true` when missing.
- `cities`: what older versions saved. When `locations` is missing, as it is for an untouched install,
  the engine searches the default locations in these cities, then every other datacenter the Windscribe
  log names in them. That is exactly what it searched before every datacenter was bundled.
- The default locations are the 20 datacenters of the old bundled list.

Datacenters come from a bundled list of every Windscribe datacenter (`src/datacenters.json`) merged with
Windscribe's client log, so a server added after the last refresh shows up once Windscribe has touched
it. `tools/update-datacenters` regenerates the list from Windscribe's public server list:

```bash
python3 -I Engine/tools/update-datacenters
```

It's run by hand before a release, and its diff is reviewed like any other change. The engine itself
never fetches the list.

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
