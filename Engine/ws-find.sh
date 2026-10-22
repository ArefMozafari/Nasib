#!/bin/zsh
# Finds a Windscribe datacenter + protocol that works right now, connects to it, and stays connected.
#   ws-find.sh          search now (tries last-known-good first)
#   ws-find.sh --watch  exit if the current tunnel works, otherwise search
#   ws-find.sh --locations print the datacenters and addresses read from the app's log
#   ws-find.sh --reach  only print which datacenters answer right now
#   ws-find.sh --survey test every combo, log each result, never stop early; Ctrl-C reconnects the last good one
#   ws-find.sh --rank   print protocols and datacenters ordered by recent success rate
set -u

STATE_DIR="$HOME/.ws-find"
LOG_FILE="$STATE_DIR/log.csv"
LAST_GOOD_FILE="$STATE_DIR/last_good"
LOCK_DIR="$STATE_DIR/lock"
BLOCKED_PROBE_URL="https://www.youtube.com/generate_204"
PROBE_TIMEOUT_SECONDS=8
# The CLI itself waits ~42s per failed attempt; a working combo answers within this.
CONNECT_TIMEOUT_SECONDS=10
# The CLI can report Connected and drop seconds later; wait before trusting it.
SETTLE_SECONDS=2
DISCONNECT_TIMEOUT_SECONDS=15

# Datacenters and their addresses are read from the app's own log at runtime, so they follow
# whatever Windscribe currently ships. Only cities in this list are used; empty it to use every city.
APP_LOG_DIR="$HOME/Library/Application Support/Windscribe/Windscribe2"
CITIES=(
  Istanbul Dubai Sofia Stockholm Amsterdam Frankfurt Zurich
  Toronto Montreal Winnipeg "New York" Chicago Atlanta
  Bratislava Helsinki Brussels
)
# Filled by load_locations_from_app_log: nicknames in log order, and nickname -> space-separated IPs.
LOCATIONS=()
typeset -A LOCATION_IPS=()
REACH_PORT=443
REACH_TIMEOUT_SECONDS=2
# Success rates come from the most recent log lines only, so the order follows current censorship, not history.
RANK_WINDOW_LINES=300
LOG_LOCATION_COLUMN=2
LOG_PROTOCOL_COLUMN=3
# Protocols in the order to try them; ranking only reorders this by recent success rate.
PROTOCOL_ORDER=(wireguard ikev2 tcp stealth wstunnel udp)
# The Nasib app writes CITIES and PROTOCOL_ORDER overrides here from its Settings tab.
CONFIG_FILE="$STATE_DIR/config"
[[ -f "$CONFIG_FILE" ]] && source "$CONFIG_FILE"

mkdir -p "$STATE_DIR"

notify() {
  # The Nasib app posts its own notifications from the printed lines and sets NASIB_NOTIFY=0.
  [[ "${NASIB_NOTIFY:-1}" == "0" ]] && return 0
  osascript -e "display notification \"$1\" with title \"Windscribe\"" >/dev/null 2>&1
}

# The app logs every datacenter it knows as:  MutableLocationInfo created:  "City - Nick" "ip; ip; "
# The same datacenter can appear with different address sets over time; all of them are kept.
load_locations_from_app_log() {
  local name ips city nickname
  grep -ohE 'MutableLocationInfo created:  \\"[^\\]+\\" \\"[0-9.; ]+' \
      "$APP_LOG_DIR/client.log" "$APP_LOG_DIR/client.1.log" 2>/dev/null \
    | sed -E 's/MutableLocationInfo created:  \\"([^\\]+)\\" \\"(.*)/\1|\2/' \
    | sort -u \
    | while IFS='|' read -r name ips; do
        city="${name%% - *}"
        nickname="${name#* - }"
        if (( ${#CITIES[@]} > 0 )) && (( ${CITIES[(Ie)$city]} == 0 )); then
          continue
        fi
        [[ -z "${LOCATION_IPS[$nickname]:-}" ]] && LOCATIONS+=("$nickname")
        LOCATION_IPS[$nickname]="${LOCATION_IPS[$nickname]:-}${ips//;/ }"
      done
  if (( ${#LOCATIONS[@]} == 0 )); then
    echo "no datacenters found in $APP_LOG_DIR; open the Windscribe app once so it refreshes its server list" >&2
    exit 1
  fi
}

# The app ignores a connect issued while it is still tearing down the previous tunnel, so wait for it.
disconnect_and_wait() {
  windscribe-cli disconnect >/dev/null 2>&1
  local waited=0
  while (( waited < DISCONNECT_TIMEOUT_SECONDS )); do
    windscribe-cli status 2>/dev/null | grep -q 'Connect state: Disconnected' && return 0
    sleep 1; (( waited++ ))
  done
  return 1
}

# Returns success only when the tunnel is up and a blocked site answers through it.
tunnel_works() {
  windscribe-cli status 2>/dev/null | grep -q 'Connect state: Connected' || return 1
  local status_code
  # --noproxy keeps the probe off any system proxy such as V2rayN.
  status_code=$(curl --noproxy '*' -m "$PROBE_TIMEOUT_SECONDS" -s -o /dev/null -w '%{http_code}' "$BLOCKED_PROBE_URL")
  [[ "$status_code" == "204" ]]
}

try_combo() {
  local location="$1" protocol="$2"
  echo "trying: $location / $protocol"
  # Non-blocking connect, then poll so a dead combo costs 10s instead of the CLI's own ~42s.
  windscribe-cli connect -n "$location" "$protocol" >/dev/null 2>&1
  local waited=0
  while (( waited < CONNECT_TIMEOUT_SECONDS )); do
    sleep 1; (( waited++ ))
    windscribe-cli status 2>/dev/null | grep -q 'Connect state: Connected' && break
  done
  sleep "$SETTLE_SECONDS"
  if tunnel_works; then
    echo "$(date +%FT%T),$location,$protocol,ok" >>"$LOG_FILE"
    echo "$location|$protocol" >"$LAST_GOOD_FILE"
    echo "connected: $location / $protocol"
    notify "Connected: $location / $protocol"
    return 0
  fi
  echo "$(date +%FT%T),$location,$protocol,fail" >>"$LOG_FILE"
  disconnect_and_wait
  return 1
}

# Prints the locations with at least one address answering on TCP 443, probed all at once (~2s total).
# A blackholed address cannot work on any protocol, so skipping it loses nothing.
reachable_locations() {
  local probe_dir location ip
  probe_dir=$(mktemp -d)
  for location in "${LOCATIONS[@]}"; do
    for ip in ${=LOCATION_IPS[$location]}; do
      ( nc -z -G "$REACH_TIMEOUT_SECONDS" -w "$REACH_TIMEOUT_SECONDS" "$ip" "$REACH_PORT" >/dev/null 2>&1 \
          && touch "$probe_dir/$location" ) &
    done
  done
  wait
  for location in "${LOCATIONS[@]}"; do
    [[ -e "$probe_dir/$location" ]] && echo "$location"
  done
  rm -rf "$probe_dir"
}

# Orders the given names by their success rate in the recent log, best first.
# Names with equal rates (including ones never tried) keep the order they were given in.
rank_by_success() {
  local column="$1"; shift
  local -a names; names=("$@")
  local rates name rate index=0
  rates=$(tail -n "$RANK_WINDOW_LINES" "$LOG_FILE" 2>/dev/null \
    | awk -F, -v col="$column" '{ total[$col]++; if ($4 == "ok") ok[$col]++ }
        END { for (name in total) printf "%s\t%.3f\n", name, ok[name] / total[name] }')
  for name in "${names[@]}"; do
    (( index++ ))
    rate=$(printf '%s\n' "$rates" | awk -F'\t' -v name="$name" '$1 == name { print $2 }')
    printf '%s\t%s\t%s\n' "${rate:-0}" "$index" "$name"
  done | sort -t $'\t' -k1,1rn -k2,2n | cut -f3
}

reconnect_last_good() {
  [[ -f "$LAST_GOOD_FILE" ]] || return 1
  local last_location last_protocol
  IFS='|' read -r last_location last_protocol <"$LAST_GOOD_FILE"
  try_combo "$last_location" "$last_protocol"
}

# Tries combos best-first. Stops at the first working one unless survey is set, in which case it
# logs every result and keeps going.
search() {
  local survey="${1:-}"
  if [[ -z "$survey" ]]; then
    reconnect_last_good && return 0
  fi
  local -a candidates protocols
  candidates=("${(@f)$(reachable_locations)}")
  if (( ${#candidates[@]} == 0 )) || [[ -z "${candidates[1]}" ]]; then
    # The filter may be wrong on a bad day; fall back to trying everything.
    echo "no address answered; trying all locations"
    candidates=("${LOCATIONS[@]}")
  fi
  candidates=("${(@f)$(rank_by_success "$LOG_LOCATION_COLUMN" "${candidates[@]}")}")
  protocols=("${(@f)$(rank_by_success "$LOG_PROTOCOL_COLUMN" "${PROTOCOL_ORDER[@]}")}")
  echo "protocol order: ${(j:, :)protocols}"
  echo "location order: ${(j:, :)candidates}"
  local location protocol
  for protocol in "${protocols[@]}"; do
    for location in "${candidates[@]}"; do
      if try_combo "$location" "$protocol"; then
        [[ -z "$survey" ]] && return 0
        disconnect_and_wait
      fi
    done
  done
  if [[ -n "$survey" ]]; then
    echo "survey finished"
    reconnect_last_good
    return $?
  fi
  echo "nothing worked"
  notify "No working server found"
  return 1
}

load_locations_from_app_log

case "${1:-}" in
  --locations)
    for nickname in "${LOCATIONS[@]}"; do echo "$nickname: ${LOCATION_IPS[$nickname]}"; done
    exit 0 ;;
  --reach)
    reachable_locations
    exit 0 ;;
  --rank)
    echo "protocols:"; rank_by_success "$LOG_PROTOCOL_COLUMN" "${PROTOCOL_ORDER[@]}"
    echo "locations:"; rank_by_success "$LOG_LOCATION_COLUMN" "${LOCATIONS[@]}"
    exit 0 ;;
esac

# One search at a time; the watcher fires every minute and a search takes longer than that.
mkdir "$LOCK_DIR" 2>/dev/null || { echo "search already running"; exit 0; }
trap 'rmdir "$LOCK_DIR" 2>/dev/null' EXIT

case "${1:-}" in
  --watch)
    tunnel_works && exit 0
    echo "tunnel down"
    notify "Tunnel down, searching..."
    search ;;
  --survey)
    # Ctrl-C mid-survey leaves the tunnel in whatever state the last attempt left it; put the user back online.
    trap 'echo; echo "survey stopped"; rmdir "$LOCK_DIR"; trap - EXIT; disconnect_and_wait; reconnect_last_good; exit 130' INT
    search survey ;;
  *)
    search ;;
esac
