#!/bin/zsh
# Runs Engine/ws-find.sh against stubbed windscribe-cli, curl and nc in a throwaway HOME,
# so engine behavior is tested without touching the real VPN. Usage: zsh Tests/Engine/ws-find.test.zsh
set -u

PROJECT_DIR="${0:A:h:h:h}"
ENGINE="$PROJECT_DIR/Engine/ws-find.sh"
LOCATION_NAME="Dubai - Khalifa"
NICKNAME="Khalifa"
PROTOCOL="ikev2"
failures=0

# Builds a sandbox: fake app log with one datacenter, stub binaries on PATH, and a tunnel state file.
# The stub mimics the real Windscribe app: a connect issued while a tunnel is up tears the old
# tunnel down and drops the new request, leaving nothing connected.
# Two marker files change the stub's behavior: "probe_hangs" makes the blocked-site check take ten
# seconds, and "probe_fails" makes it fail even when connected.
make_sandbox() {
  local sandbox
  sandbox=$(mktemp -d)
  mkdir -p "$sandbox/home/Library/Application Support/Windscribe/Windscribe2" "$sandbox/home/.ws-find" "$sandbox/bin"
  print -r -- '{"msg": "MutableLocationInfo created:  \"'"$LOCATION_NAME"'\" \"135.136.7.178; \" ; Selected node: 1"}' \
    >"$sandbox/home/Library/Application Support/Windscribe/Windscribe2/client.log"
  cat >"$sandbox/bin/windscribe-cli" <<EOF
#!/bin/zsh
state_file="$sandbox/state"
state=\$(cat "\$state_file")
case "\$1" in
  status)
    if [[ "\$state" == connected ]]; then
      print "Connect state: Connected: $LOCATION_NAME"
      print "Protocol: IKEv2:500"
    else
      print "Connect state: Disconnected"
    fi ;;
  connect)
    print "\$@" >>"$sandbox/connect_calls"
    if [[ "\$state" == connected ]]; then
      print disconnected >"\$state_file"
    else
      print connected >"\$state_file"
    fi ;;
  disconnect)
    print disconnected >"\$state_file" ;;
esac
EOF
  cat >"$sandbox/bin/curl" <<EOF
#!/bin/zsh
[[ -e "$sandbox/probe_hangs" ]] && sleep 10
[[ ! -e "$sandbox/probe_fails" && "\$(cat "$sandbox/state")" == connected ]] && print -n 204 || print -n 000
EOF
  # Limits the search to the one stub datacenter on one protocol, so a full search is a single attempt.
  print "CITIES=('Dubai')\nPROTOCOL_ORDER=($PROTOCOL)" >"$sandbox/home/.ws-find/config"
  print '#!/bin/zsh\nexit 0' >"$sandbox/bin/nc"
  chmod +x "$sandbox/bin/"*
  print "$sandbox"
}

run_engine() {
  local sandbox="$1"; shift
  HOME="$sandbox/home" PATH="$sandbox/bin:/usr/bin:/bin" NASIB_NOTIFY=0 zsh "$ENGINE" "$@" >"$sandbox/output" 2>&1
}

logged_attempts() {
  local sandbox="$1"
  [[ -f "$sandbox/home/.ws-find/log.csv" ]] || return 0
  cut -d, -f2- "$sandbox/home/.ws-find/log.csv" | paste -sd'|' -
}

expect_equal() {
  local description="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    print "ok   - $description"
  else
    print "FAIL - $description: expected [$expected], got [$actual]"
    (( failures++ ))
  fi
}

# Regression for issue #2: Find Now while connected must not burn an attempt on the last-good pair.
should_log_one_attempt_when_searching_while_connected() {
  local sandbox
  sandbox=$(make_sandbox)
  print connected >"$sandbox/state"
  print "$NICKNAME|$PROTOCOL" >"$sandbox/home/.ws-find/last_good"
  run_engine "$sandbox"
  expect_equal "search while connected logs a single attempt" \
    "$NICKNAME,$PROTOCOL,ok" "$(cut -d, -f2- "$sandbox/home/.ws-find/log.csv" | paste -sd'|' -)"
  rm -rf "$sandbox"
}

# Baseline: from a disconnected state the first attempt succeeds, as before the fix.
should_log_one_attempt_when_searching_while_disconnected() {
  local sandbox
  sandbox=$(make_sandbox)
  print disconnected >"$sandbox/state"
  print "$NICKNAME|$PROTOCOL" >"$sandbox/home/.ws-find/last_good"
  run_engine "$sandbox"
  expect_equal "search while disconnected logs a single attempt" \
    "$NICKNAME,$PROTOCOL,ok" "$(cut -d, -f2- "$sandbox/home/.ws-find/log.csv" | paste -sd'|' -)"
  rm -rf "$sandbox"
}

# The app colors output lines by type, so a failed attempt has to be printed, not only logged.
should_print_failed_line_when_an_attempt_fails() {
  local sandbox
  sandbox=$(make_sandbox)
  print disconnected >"$sandbox/state"
  touch "$sandbox/probe_fails"
  run_engine "$sandbox"
  expect_equal "a failed attempt prints a failed line" \
    "failed: $NICKNAME / $PROTOCOL" "$(grep '^failed: ' "$sandbox/output")"
  rm -rf "$sandbox"
}

# Stop in the app interrupts a find. Interrupting during the blocked-site check, the slowest step,
# must still end within a couple of seconds, disconnected, with the aborted attempt left out of the log.
should_stop_a_find_promptly_when_interrupted() {
  local sandbox engine_pid started elapsed
  sandbox=$(make_sandbox)
  print disconnected >"$sandbox/state"
  touch "$sandbox/probe_hangs"
  # A non-interactive shell starts background jobs with SIGINT ignored, and an ignored signal can't be
  # trapped. Perl resets it to the default before exec, which is how the app's Process starts the engine.
  started=$SECONDS
  HOME="$sandbox/home" PATH="$sandbox/bin:/usr/bin:/bin" NASIB_NOTIFY=0 \
    /usr/bin/perl -e '$SIG{INT} = "DEFAULT"; exec @ARGV' zsh "$ENGINE" >"$sandbox/output" 2>&1 &
  engine_pid=$!
  # Connect lands after one poll and the check starts after the two-second settle, so at four seconds
  # the engine is inside the ten-second check.
  sleep 4
  kill -INT "$engine_pid"
  wait "$engine_pid" 2>/dev/null
  elapsed=$(( SECONDS - started ))
  expect_equal "an interrupted find stops within six seconds" "yes" "$( (( elapsed <= 6 )) && print yes || print "no, took ${elapsed}s")"
  expect_equal "an interrupted find prints search stopped" "search stopped" "$(grep -x 'search stopped' "$sandbox/output")"
  expect_equal "an interrupted find logs no attempt" "" "$(logged_attempts "$sandbox")"
  expect_equal "an interrupted find leaves the tunnel disconnected" "disconnected" "$(cat "$sandbox/state")"
  rm -rf "$sandbox"
}

should_log_one_attempt_when_searching_while_connected
should_log_one_attempt_when_searching_while_disconnected
should_print_failed_line_when_an_attempt_fails
should_stop_a_find_promptly_when_interrupted
(( failures == 0 )) && print "all passed" || { print "$failures failed"; exit 1; }
