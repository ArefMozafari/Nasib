<p align="center">
  <img src="docs/images/icon.png" width="128" alt="Nasib app icon: three white waves on blue">
</p>

<h1 align="center">Nasib</h1>

<p align="center">Keeps Windscribe connected from networks that block most of it.</p>

<p align="center">
  <b>macOS</b> 15 or later, Apple silicon &nbsp;·&nbsp; <b>Windows</b> 10 and 11, x64<br>
  <a href="https://github.com/ArefMozafari/Nasib/releases/latest">Download the latest release</a>
</p>

<p align="center">
  Works with the <a href="https://windscribe.com">Windscribe</a> desktop app. Not affiliated with Windscribe.
</p>

---

Nasib (نصیب, roughly "your share") lives in the macOS menu bar or the Windows notification area. It
finds a Windscribe datacenter and protocol that actually carries traffic right now, connects to it,
and does it again when the tunnel dies.

It's for heavily filtered networks, where the server that works changes by the hour, and trying them
one by one in the Windscribe app takes longer than the connection lasts.

| macOS | Windows |
|:---:|:---:|
| <img src="docs/images/status.png" width="320" alt="macOS Status tab: connected to Dubai over IKEv2, with Find Now, Start Survey and Show Log, and colored engine output"> | <img src="docs/images/windows-status.png" width="320" alt="Windows Status tab: the current server, Find Now, Start Survey and Show Log, the two toggles, and colored engine output"> |

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

- **Needs:** macOS 15 or later on Apple silicon, and the Windscribe desktop app, logged in. Nasib
  drives its command-line tool, `/usr/local/bin/windscribe-cli`.
- **Install:** download `Nasib-<version>-macos.zip` from the
  [latest release](https://github.com/ArefMozafari/Nasib/releases/latest), unzip it, and move
  `Nasib.app` to Applications.
- **First launch:** the build isn't notarized, so macOS blocks it. Allow it under System Settings →
  Privacy & Security → Open Anyway, or clear the quarantine flag:

  ```bash
  xattr -dr com.apple.quarantine /Applications/Nasib.app
  ```

- **Launch at login:** move the app to where it will stay first. The login item points at its
  current location.

<p align="center">
  <img src="docs/images/settings.png" width="320" alt="macOS Settings tab: cities to search, and protocols in the order to try">
</p>

### Windows

- **Needs:** 64-bit Windows 10 or 11, and the Windscribe desktop app, logged in. Nasib drives its
  command-line tool, `C:\Program Files\Windscribe\windscribe-cli.exe`. The .NET runtime is inside
  the executable, so nothing else needs installing.
- **Install:** download `Nasib-<version>-win-x64.zip` from the
  [latest release](https://github.com/ArefMozafari/Nasib/releases/latest), unzip it, and put
  `Nasib.exe` in a folder where it will stay, such as `%LOCALAPPDATA%\Programs\Nasib`.
- **First launch:** the build isn't signed, so SmartScreen warns. Choose More info, then Run anyway.
- **Tray icon:** Nasib opens its window on launch, then lives in the notification area. Windows 11
  hides new icons in the overflow; to keep the waves on the taskbar, turn Nasib on under Settings →
  Personalization → Taskbar → Other system tray icons.
- **Launch at login:** starts Nasib in the notification area without the window, from wherever
  `Nasib.exe` was when you turned it on.

<p align="center">
  <img src="docs/images/windows-settings.png" width="320" alt="Windows Settings tab: cities to search, and protocols in the order to try with arrows to reorder them">
  <img src="docs/images/windows-tray-menu.png" width="166" alt="Windows tray menu: Nasib, the status line, Open Nasib, Find Now, About Nasib and Quit Nasib">
</p>

### Set Windscribe's anti-censorship options yourself

- Windscribe's Anti-Censorship preferences (Protocol Tweaks, the Amnezia config and Server Routing)
  can't be changed from its command-line tool, so Nasib uses whatever the Windscribe app has.
- On a heavily filtered network, open Windscribe's preferences once, go to Anti-Censorship, turn on
  Protocol Tweaks and pick the Amnezia config that matches your network.

## Using it

**The icon**
- The waves in the menu bar or notification area are solid when connected, dashed while searching,
  and crossed out when down.
- Clicking them opens a menu: the current server, Open Nasib, Find Now, About and Quit. On Windows,
  the menu is on right-click, and a left click opens the window.
- Closing the window doesn't quit the app.

**The controls** (hover over any of them for a one-line explanation)
- **Find Now** runs one search and stops at the first pair that works. While it runs, it reads
  **Stop**, which ends the search and disconnects.
- **Start Survey** tries every datacenter on every protocol and records each result, without
  stopping at a success. It's slow, and it's how the rankings get refreshed. **Stop Survey** ends it
  and puts you back on the last pair that worked.
- **Auto-reconnect** reads the tunnel status every 3 seconds and checks a blocked site once a minute,
  and searches when either fails.
- **Settings** holds the cities to search (one per line, as Windscribe spells them; empty means every
  city) and the protocols to try, in exactly that order.

**What it tells you**
- The output pane colors each line: green when it connects, gray for a failed attempt, orange when
  the tunnel drops, and red only when nothing works.
- Notifications come when it connects, when the tunnel drops, and when nothing works.
- **Updates:** Nasib asks GitHub for the latest release at launch and once a day. When there's a
  newer one, you get an **Update to** button next to the app name, the same item in the menu, and one
  notification. Each opens the release page; download the new zip and replace the app. Nasib never
  installs anything itself.

**Where it keeps things:** everything Nasib keeps is in `~/.ws-find` on macOS and `%LOCALAPPDATA%\Nasib` on Windows:

| File | What's in it |
|---|---|
| `log.csv` | Every attempt: time, datacenter, protocol, `ok` or `fail` |
| `last_good` | The last datacenter and protocol that worked |
| `config` (macOS), `config.json` (Windows) | What you saved in Settings |
| `preferences.json` (Windows) | Whether auto-reconnect is on, and the last update you were told about |

## Building from source

### macOS

Everything for macOS is under `macOS/`: the Swift app, the engine script, and their tests. You need
Xcode. From the repository root:

```bash
./macOS/build.sh install
```

- `./macOS/build.sh install` builds `macOS/build/Nasib.app`, copies it to `~/Applications` and
  launches it.
- `./macOS/build.sh` builds without installing.
- `./macOS/build.sh release` runs both test suites, then writes `macOS/build/Nasib-<version>-macos.zip`.

To run the tests alone:

```bash
cd macOS
swift test
zsh Tests/Engine/ws-find.test.zsh
```

- `swift test` runs the app's unit tests.
- `ws-find.test.zsh` runs the engine against stub `windscribe-cli`, `curl` and `nc` in a throwaway
  home folder, so it never touches the real VPN.
- The engine also works without the app: `macOS/Engine/ws-find.sh` with no arguments searches once,
  and it takes `--watch`, `--survey`, `--reach`, `--rank` and `--locations`, each described at the
  top of the file.

### Icons

- `Design/` at the repository root holds the icon sources both apps use: an Icon Composer document
  for the macOS app icon, which macOS 26 renders as Liquid Glass, and one SVG per menu bar state.
- `macOS/build.sh` compiles them; no PNG is exported by hand.

### Windows

Everything for Windows is under `Windows/`: a C# engine library, the WPF app, and their tests. You
need the .NET 9 SDK. From PowerShell:

```powershell
./Windows/build.ps1
```

- `build.ps1` runs the tests, then writes `Windows/build/Nasib-<version>-win-x64.zip` with a single
  self-contained `Nasib.exe`.
- `dotnet test Windows/Nasib.sln` runs the tests alone.
- The engine is a port of `macOS/Engine/ws-find.sh` with the same modes, timings and files. Its tests
  run it against a fake `windscribe-cli` and fake probes, so they never touch the real VPN.
- The tray glyph and the app icon are drawn from the same path data as the SVGs in `Design/`; the
  build writes the `.exe` icon from it with `Windows/tools/Nasib.IconBuilder`.

## Known limits

- Another VPN running a system-wide tunnel, such as v2rayN in TUN mode, can fool the blocked-site
  check and keeps auto-reconnect searching underneath it. Tracked in
  [#4](https://github.com/ArefMozafari/Nasib/issues/4).
- macOS builds are signed ad hoc, not notarized. Windows builds aren't signed.
- The Windows build is x64 only.

## License

[MIT](LICENSE). Nasib isn't affiliated with Windscribe.
