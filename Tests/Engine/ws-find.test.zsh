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
[[ "\$(cat "$sandbox/state")" == connected ]] && print -n 204 || print -n 000
EOF
  print '#!/bin/zsh\nexit 0' >"$sandbox/bin/nc"
  chmod +x "$sandbox/bin/"*
  print "$sandbox"
}

run_engine() {
  local sandbox="$1"; shift
  HOME="$sandbox/home" PATH="$sandbox/bin:/usr/bin:/bin" NASIB_NOTIFY=0 zsh "$ENGINE" "$@" >/dev/null 2>&1
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

should_log_one_attempt_when_searching_while_connected
should_log_one_attempt_when_searching_while_disconnected
(( failures == 0 )) && print "all passed" || { print "$failures failed"; exit 1; }
