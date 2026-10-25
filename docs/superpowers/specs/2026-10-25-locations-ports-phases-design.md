# Every location, chosen and ordered; ports; search phases

Status: design, not yet built. Builds on the shared Rust engine (PR #14), which isn't merged yet.

## Why

Today Nasib searches the datacenters in 16 hard-coded cities, ordered by recent success, and always on
each protocol's default port.

- **Too few locations, and no way to pick.** On a filtered network, the location that gets through is
  often one nobody would guess. Users can't see Windscribe's other locations, let alone try them in an
  order they choose.
- **Default ports only.** On the network this was tested on, ikev2 and wireguard failed on every
  datacenter, and tcp on port 443 worked on the 12th. Ports are one of the main levers against filtering,
  and the engine never tries another one.
- **The output doesn't show the port or the stage a search is in**, so a long search is hard to follow.

Success: a user can see every Windscribe datacenter, choose which ones to use and in what order, and
follow each attempt's port and the search's stages in the output. When the default ports fail, the
engine tries the other ports, and it remembers the ones that worked.

## Goals

- Every Windscribe datacenter is listed in Settings, grouped by region, with a search field.
- Users check datacenters and drag them into the order to try, as they do with protocols.
- A "Rank by recent success" switch, on by default, reorders the checked datacenters by recent success.
  Ties keep the user's order. Off, the user's order is followed exactly.
- Every attempt shows and logs its port.
- Default ports come first. Other ports are tried only when nothing worked on the default ones, and a
  port that has worked on this network is preferred afterwards.
- Every stage of a search starts with a header line in the output.

## Non-goals

- The Windows and Linux apps. They get the same Settings in their own units; the engine side is shared.
- Fetching Windscribe's server list at runtime. It may be blocked exactly where Nasib is needed.
- Telling free-account locations from Pro ones. The public list marks every datacenter as Pro, so Nasib
  can't tell; a free user's attempt on a Pro-only datacenter just fails like any other.
- The network connectivity check (#12).

## Anti-goals

- A first search that is slower than today for a user who changed nothing. With the defaults, the first
  pass must try the same datacenters, protocols and ports as today, in the same order.
- Losing the cities and protocols a user already saved, or their history in `log.csv`.
- Reading Windscribe's own preferences file or registry values to get the server list.

## The bundled list

- `Engine/src/seed_datacenters.json` is replaced by a full list generated from Windscribe's public server
  list, `https://assets.windscribe.com/serverlist/mob-v2/1/321321321`. Today that's 200 datacenters in
  127 cities, 72 countries and 75 regions.
- Each entry keeps the region (such as "Canada East"), the country code, the city, the nickname, and the
  first address of each node, which is the address the reachability knock uses today.
- `Engine/tools/update-datacenters` (a small script in the repo) downloads the list and rewrites the file.
  It's run by hand before a release; its output is reviewed in the diff like any other change. The file
  records the server list's revision it was made from.
- The Windscribe log is still merged over it, so a datacenter added after the last refresh shows up
  once Windscribe has touched it, and addresses stay current.

## Configuration

`config.json` gains two keys and keeps the existing ones:

```json
{
  "locations": ["The 6", "Khalifa", "Altstadt"],
  "rankBySuccess": true,
  "protocolOrder": ["tcp", "ikev2", "wireguard"],
  "cities": ["Toronto", "Dubai", "Zurich"]
}
```

- `locations` lists the checked datacenters by nickname, in the user's order. An empty list means every
  datacenter, in the bundled order, as an empty city list does today.
- `rankBySuccess` is `true` when missing.
- **The default locations are today's 20 seed datacenters, in today's seed order.** They're kept as
  their own list in the engine (`DEFAULT_LOCATIONS`, next to the other defaults), not derived from the
  bundled file. The full list has 36 datacenters in the 16 default cities, and its order is grouped by
  region, so deriving the defaults from it would change what a user who changed nothing searches.
  *(My call, to keep the anti-goal.)*
- **Carrying over today's settings:** when `locations` is missing, the engine uses the default
  locations whose city is in `cities`, in today's order, then every other datacenter the Windscribe log
  names in those cities, as today. An untouched install has no `locations` either, so it carries over the
  default cities and searches exactly today's list. A user who never opens the new Settings keeps
  searching what they searched before. Settings writes `locations` the first time it saves, and never
  writes an empty list or one built without the catalog. *(My call; corrected after review, which found
  the first version dropped datacenters the log adds.)*
- **No second pass when nothing answered a knock** in a find: that's a likely-offline network, and the
  other ports would only fail too. *(My call, after review.)*
- Nicknames in `locations` that aren't in the bundled list or the log (a retired datacenter) are
  skipped.

## Search order

Per run, one pass at a time. Each stage starts with a phase line.

1. **Last pair that worked**, outside a survey, exactly as today, including its port.
2. **Reachability:** the knocks, as today, over the checked datacenters.
3. **First pass:** for each protocol in the configured order, on that protocol's *preferred port*, each
   reachable datacenter in order.
   - The datacenter order is the user's order, re-sorted by recent success when `rankBySuccess` is on.
     The success rate is per datacenter, over the last 300 attempts, on any protocol and port, as today.
   - A protocol's preferred port is its best port by recent success on this network. With no history, or
     a tie, it's the protocol's default port, so a new user gets today's behavior.
4. **Second pass**, only when nothing worked in the first pass and outside a survey: for each protocol
   in order, each of its other ports in three groups:
   1. ports that have worked here, best rate first;
   2. ports never tried, in Windscribe's order;
   3. ports tried that never worked, least tried first.

   On each port, each reachable datacenter is tried in order. An untried port is more promising than
   one that has only failed, so it goes first. *(My call.)*
5. `nothing worked` only after both passes.

A survey runs stages 2 and 3 and every port of the second pass, logging each result. It's the slow,
thorough refresh it is today, now across ports too.

**Ports come from the CLI:** `windscribe-cli ports <protocol>` lists each protocol's ports, and the first
one is its default. Today, on this Mac:

| Protocol | Ports (default first) |
|---|---|
| wireguard | 443, 80, 53, 123, 1194, 65142 |
| ikev2 | 500 |
| udp | 443, 80, 53, 123, 1194, 54783 |
| tcp | 443, 587, 21, 22, 80, 123, 143, 3306, 8080, 54783, 1194 |
| stealth | 443, 587, 21, 22, 80, 123, 143, 3306, 8080, 54783, 8443 |
| wstunnel | 443 |

- They're read once per run, before the first pass.
- If the CLI returns no ports for a protocol, the engine connects without a port, as today, and logs
  the port the status then reports.
- Connecting uses `connect -n <nickname> <protocol>:<port>`.

A port's success rate is per protocol and port, across all datacenters, over the same 300-attempt window.

## Output

Attempt lines carry the port after the protocol. The apps already split the pair on the last ` / `:

- `trying: The 6 / tcp:443`
- `connected: The 6 / tcp:443`
- `failed: The 6 / tcp:443`

Phase lines start with `phase: `:

- `phase: last pair that worked`
- `phase: reachability`
- `phase: tcp on port 443`, one per protocol in the first pass
- `phase: tcp on port 587`, one per protocol and port in the second pass

The existing `protocol order (from Settings): ...` and `location order (...)` lines stay. The location
line now says whether the order came from Settings or from recent success.

## Files

- `log.csv` gains a fifth column, the port: `2026-10-25T12:51:43,The 6,tcp,ok,443`.
  - Old 4-column lines still count for datacenter ranking, and count as the default port for port ranking.
  - The status `--status` prints is unchanged.
- `last_good` becomes `<nickname>|<protocol>:<port>`. An old `<nickname>|<protocol>` file is still read,
  and the default port is used for it.

## The catalog for Settings

The apps get the list from the engine, so the bundled file has one copy:

- `nasib-engine --catalog` prints one JSON object on one line:
  `{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"The 6"}, ...]}, ...]}`.
- It lists the bundled datacenters merged with the Windscribe log. A datacenter known only from the log
  goes under a final region named "Other", since the log carries no region.
- It touches neither the tunnel nor the lock, like `--locations`.
- It joins the output contract in `Engine/README.md`, so the Windows and Linux Settings use it too.
  *(My call: one source of truth, instead of a second copy of the list in each app.)*

## Settings (macOS)

The Settings tab reads the catalog when it opens and gains a Locations section next to Protocols:

- **Your order:** the checked datacenters as rows, `City - Nickname`, each with a checkbox and a drag
  handle, like the protocol list.
- **Add locations:** a searchable list of every datacenter grouped under Windscribe's region names
  (US East, Canada East, Netherlands, ...). Checking one appends it to Your order; unchecking removes it.
- A **Rank by recent success** switch, with a one-line tooltip like the other controls.
- **Reset to Defaults** restores the default locations in their order and turns ranking on.

The old Cities text field goes away; its saved value is carried over as described above. Spacing, sizes
and colors come from the existing layout tokens, and the protocol list and the location list look and
behave the same.

The output pane shows phase lines in their own tone.

## Compatibility

- **The Windows app** still runs its C# engine and writes `config.json` without `locations`, which the
  engine handles. The Windows Settings change lands with the Windows switch to the Rust engine.
- **The Windows switch's plan** gets the same Settings as this unit, and its output parser handles the
  port and the phase lines.
- **Going back to an older version** after `log.csv` has 5-column lines: the old engines skip them
  (they expect 4 columns), so those attempts drop out of ranking. Nothing breaks. Noted for the release.

## Tests

- **Engine:**
  - the configuration fallback from `cities` to `locations`;
  - the ranking switch on and off;
  - the port ranking, with the default winning ties and untried ports last;
  - the first and second passes, the second only after nothing worked;
  - the survey covering every port;
  - the phase lines, in order;
  - the 5-column log, and old 4-column lines still read;
  - old and new `last_good`;
  - end to end: the fake CLI's `ports` command, and `protocol:port` connects.
- **macOS:**
  - parser tests for the port in attempt lines and for phase lines;
  - `EngineConfig` round-trips the new keys and keeps working with a file without them;
  - a Settings view-model test for adding, removing and reordering locations, if the logic leaves the view.
- **The real thing:** on the filtered network, a search with ikev2 and wireguard failing must reach tcp
  and say so in its phase lines, and Settings must show all 200 datacenters.

## To confirm on the real CLI first

- `connect -n <nickname> <protocol>:<port>` connects on that port, and `--status` then reports it.
- `windscribe-cli ports <protocol>` answers while no other CLI call is running. It refused with
  "Windscribe CLI is already running" while one was, so the engine reads the ports before the first
  attempt, not during one.

## Left to the builder

- How the generator script is written, as long as it runs from the repo with no extra installs on macOS.
- Whether the macOS location list is one SwiftUI view or several.
- Phase line wording beyond the `phase: ` prefix.
