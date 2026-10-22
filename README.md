<p align="center">
  <img src="docs/images/icon.png" width="128" alt="Nasib app icon: three white waves on blue">
</p>

<h1 align="center">Nasib</h1>

<p align="center">Keeps Windscribe connected from networks that block most of it.</p>

---

Nasib (نصیب, roughly "your share") lives in the macOS menu bar or the Windows notification area. It
finds a Windscribe datacenter and protocol that actually carries traffic right now, connects to it,
and does it again when the tunnel dies.

It's for heavily filtered networks, where the server that works changes by the hour, and trying them
one by one in the Windscribe app takes longer than the connection lasts.

<p align="center">
  <img src="docs/images/status.png" width="360" alt="Status tab: connected to Dubai over IKEv2, with Find Now, Start Survey and Show Log buttons">
  <img src="docs/images/settings.png" width="360" alt="Settings tab: cities to search, and protocols in the order to try">
</p>

## How it decides

1. **Reads the datacenters from Windscribe's own log**, so new, moved or retired servers show up
   without an update. Only the cities you list are used. On Windows, Windscribe logs a datacenter
   only when it connects to it, so Nasib starts from a bundled list and the log adds to it.
2. **Knocks on every address at once** and drops the ones that don't answer at all. That takes about
   two seconds and usually rules out half the list.
3. **Ranks the datacenters by recent success** over the last 300 attempts, and tries protocols
   strictly in the order you set in Settings. The last pair that worked always goes first. The
   output pane says which order came from where.
4. **Gives each attempt 10 seconds**, not the 40-odd the Windscribe CLI waits on its own.
5. **Counts a connection only when a blocked site loads through it.** Windscribe reporting
   "Connected" isn't enough; on a filtered network that often happens with nothing getting through.

## Install

### macOS

You need macOS 15 or later on Apple silicon, and the Windscribe desktop app, logged in. Nasib drives
the command-line tool that ships inside it, `/usr/local/bin/windscribe-cli`.

Download the zip from the [latest release](https://github.com/ArefMozafari/Nasib/releases/latest),
unzip it, and move `Nasib.app` to Applications. The build isn't notarized, so macOS blocks the first launch.
Allow it under System Settings → Privacy & Security → Open Anyway, or clear the quarantine flag:

```bash
xattr -dr com.apple.quarantine /Applications/Nasib.app
```

Move the app to where it will stay before turning on Launch at login. The login item points at the
app's current location.

### Windows

You need 64-bit Windows 10 or 11 and the Windscribe desktop app, logged in. Nasib drives the
command-line tool that ships with it, `C:\Program Files\Windscribe\windscribe-cli.exe`. Nothing else
needs installing; the .NET runtime is inside the executable.

Download `Nasib-<version>-win-x64.zip` from the
[latest release](https://github.com/ArefMozafari/Nasib/releases/latest), unzip it, and put
`Nasib.exe` in a folder where it will stay, such as `%LOCALAPPDATA%\Programs\Nasib`. The build isn't
signed, so SmartScreen warns on the first launch: choose More info, then Run anyway.

Nasib opens its window on launch and then lives in the notification area. Windows 11 hides new icons
in the overflow by default; to keep the waves on the taskbar, turn Nasib on under Settings →
Personalization → Taskbar → Other system tray icons. Launch at login starts it in the notification
area without the window, from wherever `Nasib.exe` was when you turned it on.

<p align="center">
  <img src="docs/images/windows-status.png" width="320" alt="Windows Status tab: the current server, Find Now, Start Survey and Show Log, the two toggles, and the engine's output">
  <img src="docs/images/windows-settings.png" width="320" alt="Windows Settings tab: cities to search, and protocols in the order to try with arrows to reorder them">
  <img src="docs/images/windows-tray-menu.png" width="200" alt="Windows tray menu: Nasib, the status line, Open Nasib, Find Now, About Nasib and Quit Nasib">
</p>

### Set Windscribe's anti-censorship options yourself

Windscribe's Anti-Censorship preferences (Protocol Tweaks, the Amnezia config and Server Routing) can't
be changed from its command-line tool, so Nasib can't set them. It uses whatever the Windscribe app has.
On a heavily filtered network, open Windscribe's preferences once, go to Anti-Censorship, turn on
Protocol Tweaks and pick the Amnezia config that matches your network.

## Using it

The waves in the menu bar or notification area are solid when connected, dashed while searching, and
crossed out when down. Clicking them gives you the current server, Open Nasib, Find Now, About and
Quit; on Windows, that menu is on right-click, and a left click opens the window. Closing the window
doesn't quit the app.

- **Find Now** runs one search and stops at the first pair that works. While it runs, the button
  reads **Stop**, which ends the search and disconnects.
- **Start Survey** tries every datacenter on every protocol and records each result without stopping
  at a success. It's slow, and it's how the rankings get refreshed. **Stop Survey** ends it and puts
  you back on the last pair that worked.
- **Auto-reconnect** checks the tunnel and a blocked site once a minute, and searches only when one of
  them fails.
- **Settings** holds the cities to search, one per line as Windscribe spells them, and which
  protocols to try. Protocols are tried in exactly that order. An empty city list means every city.

Every button and toggle explains itself in one line when you hover over it. The pane under the
toggles shows what the engine is doing, colored by kind: green when it connects, gray for an attempt
that failed, orange when the tunnel drops, and red only when nothing works.

You get a notification when it connects, when the tunnel drops, and when nothing works.

Everything Nasib keeps is in `~/.ws-find` on macOS and `%LOCALAPPDATA%\Nasib` on Windows:

| File | What's in it |
|---|---|
| `log.csv` | Every attempt: time, datacenter, protocol, `ok` or `fail` |
| `last_good` | The last datacenter and protocol that worked |
| `config` (macOS), `config.json` (Windows) | What you saved in Settings |
| `preferences.json` (Windows) | Whether auto-reconnect is on |

## Building from source

### macOS

With Xcode installed:

```bash
./build.sh install
```

That builds `build/Nasib.app`, copies it to `~/Applications` and launches it. `./build.sh` alone
builds without installing.

```bash
swift test
zsh Tests/Engine/ws-find.test.zsh
```

The first runs the app's unit tests. The second runs the engine against stub `windscribe-cli`, `curl`
and `nc` in a throwaway home folder, so it never touches the real VPN.

The engine works without the app. `Engine/ws-find.sh` with no arguments searches once; it also takes
`--watch`, `--survey`, `--reach`, `--rank` and `--locations`, each described at the top of the file.

`Design/` holds the icon sources: an Icon Composer document for the app icon, which macOS 26 renders as
Liquid Glass, and one SVG per menu bar state. `build.sh` compiles them; no PNG is exported by hand.

### Windows

Everything for Windows is under `Windows/`: a C# engine library, the WPF app, and the engine's tests.
With the .NET 9 SDK installed, from PowerShell:

```powershell
./Windows/build.ps1
```

That runs the tests, then writes `Windows/build/Nasib-<version>-win-x64.zip` with a single
self-contained `Nasib.exe`. To run the tests alone:

```powershell
dotnet test Windows/Nasib.sln
```

The engine is a port of `Engine/ws-find.sh` with the same modes, timings and files. Its tests run it
against a fake `windscribe-cli` and fake probes, so they never touch the real VPN. The tray glyph and
the app icon are drawn from the same path data as the SVGs in `Design/`; the build writes the `.exe`
icon from it with `Windows/tools/Nasib.IconBuilder`.

## Known limits

- Another VPN running a system-wide tunnel, such as v2rayN in TUN mode, can fool the blocked-site
  check and keeps auto-reconnect searching underneath it. Tracked in
  [#4](https://github.com/ArefMozafari/Nasib/issues/4).
- macOS builds are signed ad hoc, not notarized. Windows builds aren't signed.
- The Windows build is x64 only.

## License

[MIT](LICENSE). Nasib isn't affiliated with Windscribe.
