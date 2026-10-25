# Locations, Ports and Phases Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users choose and order every Windscribe datacenter, try other ports when the defaults fail, and show each attempt's port and each search stage in the output.

**Architecture:**
- The engine bundles the full datacenter list, generated from Windscribe's public server list by a repo script.
- `config.json` gains `locations` and `rankBySuccess`. The finder runs named phases: the last good pair, reachability, a first pass on preferred ports, and a second pass on the other ports.
- A new `--catalog` mode hands the list to the apps.
- The macOS Settings gets a Locations section fed by `--catalog`.

**Tech Stack:** Rust (Engine/), Swift 6 / SwiftUI (macOS/), Python 3 from the Xcode command-line tools for the generator script.

**Spec:** `docs/superpowers/specs/2026-10-25-locations-ports-phases-design.md`

## Global Constraints

- Base: branch `feature/locations-and-ports`, cut from `feature/rust-engine` (PR #14, not merged). Rebase onto `main` once #14 merges.
- **Server list source:** `https://assets.windscribe.com/serverlist/mob-v2/1/321321321`. Fetched only by the generator script, never at runtime.
- **Default locations:** today's 20 seed nicknames, in today's order:
  > Lygos, Khalifa, Vitoshka, Nevski, Fika, Djurgarden, Stolen Cheese, Wurstchen, Sausage Party, Altstadt, The 6, Bagel Poutine, The Forks, Inside Job, Green Mill, Magic City, Devin Castle, Sauna, Guildhouse, Tram

  Read the exact list and order from the current `Engine/src/seed_datacenters.json` before it is replaced.
- **Output lines:**
  - attempts: `trying: <nickname> / <protocol>:<port>`, and the same for `connected:` and `failed:`;
  - phases start with `phase: `.
- `log.csv` line: `YYYY-MM-DDTHH:MM:SS,<nickname>,<protocol>,ok|fail,<port>`. Old 4-column lines stay valid.
- `last_good`: `<nickname>|<protocol>:<port>`. Old `<nickname>|<protocol>` stays valid.
- **Windscribe files:** never read Windscribe's preferences file or registry values.
- **Repo rules:**
  - full-word identifiers, no magic values;
  - comments above the code, as full sentences;
  - helpers defined above their callers;
  - one test file per source file, test names `should_...` (Rust) and `testShould...` (Swift);
  - UI sizes in `Layout`, hover text in `Tooltip`.
- **Commits:** small and split by concern, in the house style (`feat:`, `docs:`, `refactor:`, `test:`, `build:`), with no attribution. Nothing is committed or pushed without the user's go-ahead.

## Review Focus

1. **`locations` naming nicknames that are neither bundled nor in the log:** they're skipped. If none is left, the run prints `no datacenters match the locations in Settings` and fails without connecting. Pinned in Task 4.
2. **`windscribe-cli ports` printing nothing or garbage, or being busy:** that protocol connects without a port, as today, and its attempts log the port the status reports. Pinned in Task 6.
3. **`log.csv` mixing 4- and 5-column lines:** both count for datacenter ranking, and 4-column lines count as the protocol's default port. Pinned in Task 7.
4. **A stop in the middle of the second pass or a port survey:** the same stop behavior as today. A find disconnects; a survey reconnects `last_good`, including its port. Pinned in Task 8.
5. **The catalog read when the CLI is missing:** `--catalog` needs no CLI and still prints the list, so Settings works before Windscribe is installed. Pinned in Task 9.

---

### Task 1: Confirm the CLI's port behavior

This is a verification task, with no code. It changes the tunnel, so run it once the user is ready.

- [ ] **Step 1:** Confirm that the ports are available when nothing else is running: `windscribe-cli ports tcp` prints `443, 587, 21, 22, 80, 123, 143, 3306, 8080, 54783, 1194`.
- [ ] **Step 2:** Run `windscribe-cli connect -n "The 6" tcp:587`, poll `windscribe-cli status`, and see `Protocol: TCP:587` (or a failure, which is fine). Then run `windscribe-cli connect -n "The 6" tcp:443` to put the user back.
- [ ] **Step 3:** Ledger what the CLI actually printed. If `protocol:port` isn't accepted, stop and ask; the port design depends on it.

### Task 2: The bundled datacenter list and its generator

**Files:**
- Create: `Engine/tools/update-datacenters` (Python 3, executable, no third-party modules, run with `python3 -I`)
- Replace: `Engine/src/seed_datacenters.json` with the generated file, renamed `Engine/src/datacenters.json`
- Modify: `Engine/src/datacenter.rs`, `Engine/src/catalog.rs`
- Test: `Engine/tests/datacenter.rs`, `Engine/tests/catalog.rs`

**Interfaces:**
- **File shape:** `{ "revision": <number>, "datacenters": [{ "region": "Canada East", "countryCode": "CA", "city": "Toronto", "nickname": "The 6", "addresses": ["198.44.157.19", ...] }, ...] }`, in the server list's order. Each address is the node's `ip`.
- **`Datacenter`:** gains `pub region: Option<String>` and `pub country_code: Option<String>`. Both are `None` for datacenters known only from the Windscribe log. `from_location_name` leaves them `None`.
- **`catalog::bundled() -> Vec<Datacenter>`:** replaces `seed()`.
- **`catalog::merge(bundled, logged) -> Vec<Datacenter>`:** no longer filters by configuration. Selection moves to `Configuration::select` (Task 4).

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_load_every_bundled_datacenter_with_a_region`: count ≥ 200, unique nicknames, all regions `Some`;
  - `should_include_every_default_location`: every nickname in `DEFAULT_LOCATIONS` (Task 4) is bundled;
  - `should_keep_the_region_when_the_log_adds_addresses`;
  - `should_add_log_only_datacenters_without_a_region`.
- [ ] **Step 2:** Run `cargo test --test catalog --test datacenter`. It fails.
- [ ] **Step 3:** Write `Engine/tools/update-datacenters`. It downloads the URL, keeps the fields above, writes 2-space-indented JSON with one datacenter per line, and prints the counts. Run it to produce `datacenters.json`.
- [ ] **Step 4:** Implement. Update `finder.rs` and `main.rs` to call `catalog::bundled()` and the new `merge`; selection is temporarily the old city filter until Task 4.
- [ ] **Step 5:** `cargo test` passes in full.
- [ ] **Step 6:** Commits:
  - `build: add a script that regenerates the bundled datacenter list`;
  - `feat: bundle every Windscribe datacenter with its region`.

### Task 3: Ports in attempts, the log and last_good

**Files:**
- Modify: `Engine/src/attempt_log.rs`, `Engine/src/events.rs`
- Test: `Engine/tests/attempt_log.rs`, `Engine/tests/events.rs`

**Interfaces:**
- **`ServerPair`:** gains `pub port: Option<u16>`.
  - `ServerPair::label(&self) -> String` returns `<protocol>:<port>`, or `<protocol>` when the port is `None`.
  - Events print `<nickname> / <label>`.
- **`AttemptRecord`:** gains `pub port: Option<u16>`.
- **`AttemptLog::record(&self, pair: &ServerPair, succeeded: bool)`:** writes 5 columns. A `None` port writes an empty fifth column.
- **`parse_record`:** accepts 4 or 5 columns. A 4th column other than `ok`/`fail` makes the line malformed, which also fixes the deferred minor about partial `ok`.
- **`read_last_good`:** accepts `nick|protocol:port` and `nick|protocol`.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_append_the_port_as_a_fifth_column`;
  - `should_read_four_and_five_column_lines`;
  - `should_skip_a_line_whose_result_is_cut_off`;
  - `should_round_trip_last_good_with_a_port`;
  - `should_read_last_good_without_a_port`;
  - `should_print_the_port_after_the_protocol`, e.g. `trying: The 6 / tcp:443`;
  - `should_print_the_protocol_alone_without_a_port`.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement. Update the callers in `finder.rs` to pass the pair. Ports stay `None` everywhere until Task 6.
- [ ] **Step 4:** `cargo test` passes in full.
- [ ] **Step 5:** Commit `feat: record the port of every attempt`.

### Task 4: Locations and the ranking switch in the configuration

**Files:**
- Modify: `Engine/src/configuration.rs`
- Test: `Engine/tests/configuration.rs`

**Interfaces:**
- **`pub const DEFAULT_LOCATIONS: [&str; 20]`**, in the order listed under Global Constraints.
- **`Configuration`:**
  - gains `pub locations: Option<Vec<String>>`, where `None` means the key was missing;
  - gains `pub rank_by_success: bool`, `true` when missing;
  - keeps `cities`.
  - The default has `locations: Some(DEFAULT_LOCATIONS)` and `rank_by_success: true`.
- **`Configuration::select(&self, datacenters: &[Datacenter]) -> Vec<Datacenter>`:**
  - `Some(list)`, non-empty: those nicknames in list order, skipping unknown ones;
  - `Some(empty)`: every datacenter in catalog order;
  - `None`: the `DEFAULT_LOCATIONS` whose city `includes_city`, in that order, then any other datacenter in an included city that has no region (it came from the log only), in catalog order.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_select_todays_twenty_with_the_default_cities`;
  - `should_select_default_locations_in_saved_cities_when_locations_is_missing`;
  - `should_append_log_only_datacenters_in_saved_cities`;
  - `should_follow_the_saved_locations_order`;
  - `should_skip_unknown_locations`;
  - `should_select_every_datacenter_when_locations_is_empty`;
  - `should_rank_by_success_when_the_key_is_missing`;
  - `should_read_rank_by_success_off`.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement. `finder.rs` uses `configuration.select(&merged)`. When nothing is selected, it prints `no datacenters match the locations in Settings` (Review Focus 1).
- [ ] **Step 4:** Run the tests and see them pass; then `cargo test` in full.
- [ ] **Step 5:** Commit `feat: choose and order the datacenters to search`.

### Task 5: Ranking switch and port ranking

**Files:**
- Modify: `Engine/src/ranking.rs`
- Test: `Engine/tests/ranking.rs`

**Interfaces:**
- `rank(names, recent)` is unchanged; the finder calls it only when `rank_by_success` is on.
- `pub struct PortOrder { pub preferred: Option<u16>, pub others: Vec<u16> }`.
- `pub fn rank_ports(protocol: &str, ports: &[u16], recent: &[AttemptRecord]) -> PortOrder`:
  - **Default port:** `ports[0]`. A record without a port counts as the default port.
  - **`preferred`:** the port with the best success rate among those that have worked. With no successes, it's the default.
  - **`others`:** the remaining ports in three groups:
    1. worked, best rate first;
    2. never tried, in the given order;
    3. tried and never worked, least tried first.
  - Ties keep the given order.
  - Empty `ports` give `preferred: None` and `others: []`.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_prefer_the_default_port_without_history`;
  - `should_prefer_a_port_that_worked_over_the_default`;
  - `should_count_records_without_a_port_as_the_default`;
  - `should_put_untried_ports_before_ports_that_only_failed`;
  - `should_put_less_tried_failing_ports_first`;
  - `should_return_no_port_when_the_cli_lists_none`;
  - `should_ignore_other_protocols`.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and see them pass.
- [ ] **Step 5:** Commit `feat: rank each protocol's ports by what worked`.

### Task 6: Ports from the CLI and connecting on a port

**Files:**
- Modify: `Engine/src/windscribe_cli.rs`, `Engine/src/tunnel.rs`, `Engine/tests/support/mod.rs`, `Engine/tests/support/fake-windscribe-cli`
- Test: `Engine/tests/windscribe_cli.rs`, `Engine/tests/tunnel.rs`

**Interfaces:**
- **`WindscribeCli` trait:**
  - gains `fn ports(&self, protocol: &str) -> String`;
  - `connect(&self, nickname: &str, protocol: &str, port: Option<u16>)` passes `protocol:port` when the port is `Some`.
- **`windscribe_cli::parse_ports(output: &str) -> Vec<u16>`:** reads the comma-separated numbers on the first line that has only digits, commas and spaces, which skips the JSON log lines. Anything else gives an empty list.
- **`connect_arguments(nickname, protocol, port)`.**
- **`TunnelStatus`:** gains `pub port: Option<u16>`, parsed from `Protocol: TCP:443`.
- **Fakes:**
  - `FakeCli` gains `ports_by_protocol: HashMap<String, String>` and records `connect <nickname> <protocol>:<port>`.
  - The fake script answers `ports`, from `$FAKE_CLI_DIR/ports-<protocol>` when that file exists.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_parse_the_ports_after_the_log_lines`;
  - `should_return_no_ports_for_unexpected_output`;
  - `should_return_no_ports_when_the_cli_is_busy` (input: `Windscribe CLI is already running`);
  - `should_connect_on_the_given_port`;
  - `should_connect_without_a_port_when_none_is_given`;
  - `should_read_the_port_from_the_status`.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test` passes in full.
- [ ] **Step 5:** Commit `feat: read each protocol's ports and connect on a chosen port`.

### Task 7: Phases, the two passes and the order lines

**Files:**
- Modify: `Engine/src/finder.rs`, `Engine/src/events.rs`
- Test: `Engine/tests/finder.rs`, `Engine/tests/events.rs`

**Interfaces:**
- **`EngineEvent`:** gains `Phase(String)`, printed as `phase: <text>`.
- **Phase texts:**
  - `last pair that worked`;
  - `reachability`;
  - `<protocol> on port <port>`, or `<protocol>` when there is no port.
- **Location order line:**
  - `location order (from Settings, by recent success): …` when ranking is on;
  - `location order (from Settings): …` when it's off.
- **Finder, per run:**
  - reads every protocol's ports once, with `cli.ports`, through a new `Tunnel::ports(protocol) -> Vec<u16>`;
  - uses `rank_ports` on the recent log;
  - first pass: each protocol on its `preferred` port;
  - second pass, only when the first pass found nothing and outside a survey: each protocol's `others`, then `nothing worked`;
  - a survey runs the first pass and then every port in `others`.
- **Logged ports:** a pair with no port logs the port `read_status()` reports after connecting (Review Focus 2).

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_print_the_phases_in_order`;
  - `should_try_the_preferred_port_first`;
  - `should_try_other_ports_only_after_the_first_pass_fails`;
  - `should_skip_the_second_pass_when_the_first_pass_connects`;
  - `should_order_locations_from_settings_when_ranking_is_off`;
  - `should_rank_locations_when_ranking_is_on`;
  - `should_count_four_column_history_for_the_default_port`;
  - `should_log_the_status_port_when_the_cli_lists_no_ports`;
  - `should_survey_every_port`;
  - `should_reconnect_last_good_on_its_port_after_a_stopped_survey`;
  - `should_stop_during_the_second_pass_like_a_find`;
  - `should_search_todays_pairs_first_with_default_settings`, the anti-goal: default configuration and no history give the same first-pass pairs and order as before this unit.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test` passes in full, and `cargo clippy --all-targets -- -D warnings` is clean.
- [ ] **Step 5:** Commit `feat: search in phases and try other ports when the defaults fail`.

### Task 8: End to end with ports

**Files:**
- Modify: `Engine/tests/main.rs`

- [ ] **Step 1:** Write the failing tests:
  - `should_print_the_port_and_phases`: with `ports-ikev2` holding `500`, the output contains `phase: ikev2 on port 500` and `trying: Fake / ikev2:500`;
  - `should_log_five_columns`;
  - `should_reconnect_last_good_with_its_port_when_the_app_quits_mid_survey`.
- [ ] **Step 2:** Run them and see them fail, then make them pass. Change the fake script only if needed.
- [ ] **Step 3:** Commit `test: cover ports and phases end to end`.

### Task 9: `--catalog`

**Files:**
- Modify: `Engine/src/mode.rs`, `Engine/src/main.rs`, `Engine/README.md`
- Create: `Engine/src/catalog_output.rs`
- Test: `Engine/tests/mode.rs`, `Engine/tests/catalog_output.rs`, `Engine/tests/main.rs`

**Interfaces:**
- **`Mode::Catalog`**, from `--catalog`.
- **`catalog_output::render(datacenters: &[Datacenter]) -> String`:** one JSON line, `{"regions":[{"name":…,"datacenters":[{"city":…,"nickname":…}]}]}`.
  - Regions keep their first-appearance order.
  - Datacenters with no region go under a final `Other`.
- **CLI not needed:** `--catalog` runs before the CLI check, so it works without Windscribe (Review Focus 5).

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `should_group_by_region_in_order`;
  - `should_put_log_only_datacenters_under_other`;
  - `should_print_the_catalog_without_the_cli`, end to end with a missing CLI path.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement. In `Engine/README.md`, document:
  - `--catalog` and its JSON;
  - the port in attempt lines;
  - phase lines;
  - the 5-column log;
  - `last_good` with a port;
  - `locations` and `rankBySuccess` in `config.json`.
- [ ] **Step 4:** `cargo test` passes in full.
- [ ] **Step 5:** Commits:
  - `feat: print the datacenter catalog for Settings`;
  - `docs: document locations, ports and phases in the engine README`.

### Task 10: macOS parser and tones

**Files:**
- Modify: `macOS/Sources/Nasib/EngineOutputParser.swift`, `macOS/Sources/Nasib/ContentView.swift` (the `OutputTone.color` mapping)
- Test: `macOS/Tests/NasibTests/EngineOutputParserTests.swift`

**Interfaces:**
- **`EngineEvent`:** gains `.phase(String)` with a new `OutputTone.phase`, colored `.teal` in `OutputTone.color`.
- **Attempt lines:** keep their `protocolName` as the engine prints it (`tcp:443`). The menu shows `The 6 / tcp:443`.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `testShouldParsePhaseLine`;
  - `testShouldKeepThePortInTryingLine`;
  - `testShouldGivePhasesTheirTone`.
- [ ] **Step 2:** Run `swift test --filter EngineOutputParserTests`. It fails.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** The filtered tests pass.
- [ ] **Step 5:** Commit `feat: show phase lines in their own color on macOS`.

### Task 11: macOS config and catalog model

**Files:**
- Modify: `macOS/Sources/Nasib/EngineConfig.swift`
- Create: `macOS/Sources/Nasib/EngineCatalog.swift`, `macOS/Sources/Nasib/LocationSelection.swift`
- Test:
  - `macOS/Tests/NasibTests/EngineConfigTests.swift`;
  - `macOS/Tests/NasibTests/EngineCatalogTests.swift`;
  - `macOS/Tests/NasibTests/LocationSelectionTests.swift`.

**Interfaces:**
- **`EngineConfig`:**
  - gains `var locations: [String]?` and `var rankBySuccess: Bool`, both `Codable`;
  - a missing `rankBySuccess` decodes as `true`;
  - `cities` stays, for the carry-over;
  - `static let defaultLocations`, the same 20 in the same order.
- **`EngineCatalog`:**
  - `struct EngineCatalog: Decodable { let regions: [Region] }`, with `Region { name, datacenters: [Entry] }` and `Entry { city, nickname }`;
  - `static func decode(_ line: String) -> EngineCatalog?`;
  - `Engine` gains `nonisolated static func readCatalog() -> EngineCatalog?`, which runs `nasib-engine --catalog`.
- **`LocationSelection`,** a value type for Settings:
  - `init(saved: EngineConfig, catalog: EngineCatalog)`. A config with no `locations` starts from the same selection the engine would make: the default locations in the saved cities;
  - `ordered: [EngineCatalog.Entry]`;
  - `func isSelected(_ nickname) -> Bool`;
  - `mutating func toggle(_ nickname)`, which appends or removes;
  - `mutating func move(from: IndexSet, to: Int)`;
  - `mutating func resetToDefaults()`;
  - `var nicknames: [String]`.

Steps:
- [ ] **Step 1:** Write the failing tests:
  - `testShouldRoundTripLocationsAndRanking`;
  - `testShouldDefaultRankingOnWhenMissing`;
  - `testShouldDecodeTheCatalogLine`;
  - `testShouldStartFromDefaultLocationsInSavedCities`;
  - `testShouldAppendWhenToggledOn`;
  - `testShouldRemoveWhenToggledOff`;
  - `testShouldMoveARow`;
  - `testShouldResetToTheDefaultOrder`.
- [ ] **Step 2:** Run the tests and see them fail.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run the tests and see them pass.
- [ ] **Step 5:** Commit `feat: model the location choice and the catalog on macOS`.

### Task 12: macOS Settings: Locations section

**Files:**
- Modify: `macOS/Sources/Nasib/SettingsView.swift`, `macOS/Sources/Nasib/Tooltip.swift`, the `Layout` tokens in `macOS/Sources/Nasib/ContentView.swift`
- Create: `macOS/Sources/Nasib/LocationsSettings.swift`

**Layout:**
- The cities editor is replaced by a Locations section:
  - **Your order:** a `List` of `LocationSelection.ordered` with toggles and `.onMove`, like the protocol list, `Layout.locationOrderHeight`;
  - **Add locations:** a search field, then a `List` with a `Section` per region and a toggle per datacenter, `Layout.locationPickerHeight`;
  - **Rank by recent success:** a toggle.
- A catalog that can't be read shows a red caption, "Couldn't read the datacenter list," and keeps the saved order editable.
- **New `Tooltip` entries:**
  - `rankBySuccess`: "Try the checked locations in the order that worked best lately, instead of exactly your order.";
  - `save` and `resetToDefaults` say "locations" instead of "cities".
- **Visual check:** before and after screenshots of the Settings tab for the PR. Capture the "before" from the `feature/rust-engine` build first.

Steps:
- [ ] **Step 1:** Capture the "before" screenshot of Settings from the currently installed build.
- [ ] **Step 2:** Implement. Save writes `locations`, `rankBySuccess` and `protocolOrder`, and keeps `cities` as loaded.
- [ ] **Step 3:** `swift test` passes in full. Run `./macOS/build.sh install`, open Settings, and confirm:
  - all regions are listed;
  - search filters the list;
  - checking a datacenter adds it to Your order;
  - dragging reorders it;
  - Save writes `config.json`.
- [ ] **Step 4:** Capture the "after" screenshot.
- [ ] **Step 5:** Commit `feat: choose and order locations in macOS Settings`.

### Task 13: Verify on the real network

- [ ] **Step 1:** Back up `~/.ws-find`.
- [ ] **Step 2:** With the default settings, check that Find Now prints the phases and attempt lines carry ports:
  - its first pass tries the same pairs as before;
  - a failing protocol is followed by the next one, and its phase line is printed;
  - when nothing works on the default ports, the second pass starts with the ports never tried.
- [ ] **Step 3:** Check from Settings:
  - checking a datacenter outside the defaults and moving it to the top makes it the first one tried after the last good pair;
  - with ranking off, the order is followed exactly.
- [ ] **Step 4:** Ledger the observations, with times.

### Task 14: Docs

**Files:**
- Modify: `README.md`

- [ ] **Step 1:** Update the README:
  - "How it decides": the user's locations, the ranking switch, phases, and the second pass on other ports;
  - "Using it": the Locations section replaces the cities;
  - the files table: the port column, and `last_good` with a port;
  - "Known limits": Pro-only datacenters aren't marked; going back to an older version drops 5-column history from ranking.
- [ ] **Step 2:** Commit `docs: describe locations, ports and phases`.
- [ ] **Step 3:** Stop. Report, and wait for approval to push and open the PR.
