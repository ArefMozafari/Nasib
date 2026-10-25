# Nasib on Linux

Status: design, not yet built.

## Why

Nasib runs on macOS and Windows, both on the shared Rust engine (`Engine/`, #14 and #19). Linux users who
need Windscribe on a filtered network have nothing. The engine already builds and passes its tests on
Linux; what's missing is an app around it.

Success: a Linux user on one of the target distros installs one package, gets a tray icon and a window
that behave like the other two apps, and Nasib keeps their Windscribe tunnel working the same way.

## Goals

- A GTK 4 app in Rust, in a new `Linux/` folder, with the same two tabs, tray menu and behavior as the
  macOS and Windows apps.
- It runs `nasib-engine` as a child process and reads its output lines, exactly as the other apps do.
- Packages for x64: `.deb`, `.rpm`, and a `.tar.gz` for other distros.
- Verified on a real Windscribe connection, in an ARM Ubuntu VM on the Mac.

## Non-goals

- AppImage, Flatpak or Snap. Bundling GTK 4 into them is large and fragile; distro packages pull it in.
- ARM release packages. The ARM build exists only to test in the VM.
- Distros older than the targets below, and desktops without a tray (the window still works there).
- A Linux-specific feature the other apps don't have.

## Anti-goals

- Behavior that differs from the other two apps: search, stop, quit, auto-reconnect, settings rules and
  update checks must match.
- Reading Windscribe's own settings files on Linux, as on the other platforms.
- A second copy of the engine's logic in the app. The app parses output lines and writes `config.json`;
  the engine decides everything else.

## Targets

| Distro | GTK 4 it ships | Tray out of the box |
|---|---|---|
| Ubuntu 22.04 and later | 4.6+ | Yes (Ubuntu's GNOME has AppIndicator support) |
| Debian 12 | 4.8 | GNOME needs the AppIndicator extension; KDE and Xfce yes |
| Fedora 38 and later | 4.10+ | GNOME needs the AppIndicator extension; KDE yes |
| Linux Mint 21 and later | 4.6+ | Yes (Cinnamon) |

- The app needs GTK 4.6 or later, so it uses plain GTK 4 and no libadwaita, which Ubuntu 22.04 has only
  in an old version. *(My call.)*
- Binaries are built on Ubuntu 22.04 (glibc 2.35), the oldest target, so they run on every newer one.
- Without a tray, the window opens at launch and closing it quits the app, instead of hiding to a tray
  that isn't there. *(My call.)*
- **Launching:** a launch by hand opens the window, as on Windows. The login entry passes `--background`,
  so launch at login only adds the tray icon, as its tooltip says. A second launch brings the running
  app's window forward instead of starting another app. *(My call.)*
- **Dark mode:**
  - Plain GTK 4.6–4.10 doesn't follow GNOME's dark style on Fedora and Debian, since only libadwaita reads
    GNOME's `color-scheme` setting.
  - So the app reads the desktop's light or dark preference from the standard desktop settings portal
    (`org.freedesktop.appearance color-scheme`) and sets GTK's prefer-dark setting to match, live.
  - On Ubuntu and KDE, which already switch the theme, this is a no-op. *(The user's decision: follow the
    desktop.)*

## How it's built

### The engine and its files

- The app runs `nasib-engine` from the same folder as its own binary, as the other apps do.
- The packages install both binaries to `/usr/lib/nasib/`, and a launcher `/usr/bin/nasib` runs the app.
  *(My call.)*
- The state folder is the engine's Linux default, `$XDG_DATA_HOME/nasib` (`~/.local/share/nasib`). The
  app keeps its own `preferences.json` there, as the Windows app does: auto-reconnect and the last update
  it notified.
- The app links the engine crate only for its shared constants: default cities, default locations, all
  protocols and the state folder path. It never calls the engine's search code. *(My call: one source for
  the defaults, and no second engine.)*
- Windscribe's CLI and log locations on Linux are still unconfirmed (`Engine/README.md` marks them so).
  The first step in the VM confirms them, and the engine's Linux defaults are corrected if they're wrong.
- If the Linux CLI also differs in behavior, such as its `status` text, its `ports` listing or a helper
  that must be running, the engine fixes land in this unit as their own tested commits, before the app.
  They only add Linux cases, so the macOS and Windows behavior doesn't change. *(My call.)*

### Behavior, matching the other apps

- **Status tab:**
  - the app icon and name, with an "Update to X" button when a newer release exists;
  - the server line with its status dot;
  - Find Now (Stop while finding), Start Survey (Stop Survey), Show Log (opens `log.csv` with the
    default app);
  - Auto-reconnect and Launch at login switches;
  - the output pane: the last 200 lines in their tones, phase lines teal, alternating rows striped by
    line number.
- **Settings tab:** the same two boxes as macOS:
  - **Locations:** Your order and Add locations side by side, each with a search field and Select all /
    Deselect all, a link per region, the ranking switch, and "N of M checked";
  - **Protocols:** the list with Select all / Deselect all;
  - Reset to Defaults and Save, with the same save rules and messages.

  GTK lists reorder by dragging, as on macOS. Dragging is off while Your order is filtered.
- **Tray menu:** the status line, Open Nasib, Find Now / Stop Search, "Update to X…" when there is one,
  About Nasib, Quit Nasib.
- **The engine:**
  - status every 3 seconds through `--status`;
  - the once-a-minute check through `--watch` when auto-reconnect is on, and at once when a connected
    tunnel drops;
  - Stop sends `stop` on the engine's standard input;
  - quitting closes it, so the engine finishes its clean-up on its own.
- **Notifications** through the desktop's notification service: connected, tunnel down, nothing worked,
  and one per new release.
- **Launch at login** writes `~/.config/autostart/nasib.desktop`, which every target desktop honors.
- **Update check:** GitHub's latest release at launch and once a day, as the other apps do. It only opens
  the release page.
- **Strings and tooltips:** the same strings as the other two apps.

### Look

- Native GTK 4 widgets with the desktop's own theme, light or dark.
- The three tray glyphs come from `Design/`'s SVGs, installed as hicolor icons.
- The app icon's only source today is the Icon Composer document, which a Linux build can't compile. So
  `Design/AppIcon.svg` is added: the blue square with the three waves, drawn from the same path data the
  tray glyphs and the Windows icon use. It's installed as the scalable hicolor app icon. *(My call.)*
- A small `Layout`-style constants module holds sizes and spacing, like the other apps' token layers.
- The two Settings boxes are GTK frames, the counterpart of macOS group boxes.

## Packaging

- `Linux/build.sh` builds and tests in a Docker `ubuntu:22.04` container with GTK 4's development
  packages, for x64.
- In `release` mode it writes `Nasib-<version>-linux-x64.deb`, `Nasib-<version>-linux-x64.rpm` and
  `Nasib-<version>-linux-x64.tar.gz`.
  - The packages depend on the distro's GTK 4 and declare Windscribe as a recommended package, not a
    hard dependency, since Windscribe's own package names differ by distro.
  - The tarball holds both binaries, the desktop file and icons, and an `install.sh` that copies them
    under `~/.local`.
- The version comes from one place, `Linux/Cargo.toml`, bumped with the other two apps.

## Testing

- **Unit tests in Rust**, one test file per source file, for everything that isn't drawing. The macOS
  and Windows tests are ported for the same rules:
  - the output parser, tones and status line;
  - the catalog and the location selection: carry-over, select / deselect, region links, reset,
    filtered order;
  - the configuration and its save rules;
  - preferences;
  - the controller: tunnel drop, stripes, the busy and finding states;
  - the update check's version compare;
  - the login item file;
  - the engine process: lines, stop, input closing on quit, against a small stub script.
- **The real thing in the VM:** the checks the other two apps passed, plus the tray on Ubuntu's GNOME:
  - status every 3 s;
  - Find Now;
  - Stop during a find and a survey;
  - quit mid-survey;
  - auto-reconnect after a manual disconnect;
  - Settings carry-over and save;
  - notifications;
  - launch at login after a re-login.

## The VM

- UTM on this Mac, with Ubuntu 24.04 Desktop for ARM and Windscribe's ARM Linux app.
- The user logs in to Windscribe inside it once.
- Installing UTM and downloading the Ubuntu image (about 4 GB together) wait for the user's go-ahead.
- The VM can't test x64 packages. The ARM build goes into the VM, and the x64 packages are checked by
  installing them in an x64 container: they install, and `nasib-engine --catalog` runs.
- **Fallback, if Windscribe's ARM app doesn't work in the ARM VM:** an emulated x64 Ubuntu VM in UTM,
  which also runs the real x64 `.deb`. It's slow, and it's another download of about 6 GB, which the
  user approved with this choice.

## Left to the builder

- Module layout inside `Linux/src`, within the repo's rules.
- Which GTK list widgets carry the location lists, as long as dragging, search and region links work.
- How `.deb` and `.rpm` are produced (for example `cargo-deb` and `cargo-generate-rpm`).
