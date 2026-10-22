# Nasib

A macOS menu bar app that finds a Windscribe datacenter and protocol that works from a censored
network, keeps you connected, and reconnects on its own when the tunnel dies.

## How it works

- `Engine/ws-find.sh` is the engine. It reads the datacenters Windscribe currently ships from the
  Windscribe app's own log, probes which ones answer at all, ranks datacenters and protocols by
  their recent success rate, and connects through `windscribe-cli`. A connection counts only when a
  blocked site answers through the tunnel.
- The app bundles that script, runs it, and shows its progress. With auto-reconnect on, it runs the
  script's watch mode every minute, which does nothing while the tunnel is healthy.
- State lives in `~/.ws-find`: `log.csv` holds every attempt, `last_good` the last working pair.

## Build and run

Requires Xcode and the Windscribe desktop app with its CLI at `/usr/local/bin/windscribe-cli`.

```
./build.sh install
```

Builds `build/Nasib.app`, copies it to `~/Applications`, and launches it. Run `./build.sh` alone to
build without installing. `swift test` runs the unit tests.

## Using it

The menu bar icon shows the state: shield with a check when connected, arrows while searching, a
crossed shield when down. The window has Find Now, Start/Stop Survey (tests every combination and
records each result without stopping), Show Log, and toggles for auto-reconnect and launch at login.
Closing the window keeps the app in the menu bar; quit from there.

Cities to search are the `CITIES` list in `Engine/ws-find.sh`.
