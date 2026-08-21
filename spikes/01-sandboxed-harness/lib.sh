# Shared setup for the three credential candidates. Sourced, not run.
#
# The "real credential" is whatever the operator supplies in LOCUS_SPIKE_REAL_KEY.
# When none is supplied a synthetic one is generated for this invocation: the
# exposure question — what can be read out of the container — is answered
# identically either way, and the mock upstream accepts exactly that value.
#
# No script here ever prints a credential. Everything is reported by class.
set -euo pipefail

SPIKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$SPIKE_DIR/out"
mkdir -p "$OUT"

: "${LOCUS_PROXY_PORT:=43800}"
: "${LOCUS_MOCK_PORT:=43801}"
: "${LOCUS_EGRESS_TIER:=model}"
export LOCUS_PROXY_PORT LOCUS_MOCK_PORT LOCUS_EGRESS_TIER

if [ -z "${LOCUS_SPIKE_REAL_KEY:-}" ]; then
  export LOCUS_SPIKE_REAL_KEY="sk-synthetic-$(openssl rand -hex 24)"
  export LOCUS_KEY_ORIGIN="synthetic"
else
  export LOCUS_KEY_ORIGIN="operator-supplied"
fi

export LOCUS_SENTINEL="${LOCUS_SENTINEL:-sk-locus-sentinel-00000000000000000000000000000000}"

LOCUSD_PID=""
LOCUS_SOCK_PATH=""

start_locusd() {
  local tag="$1"
  LOCUS_SOCK_PATH="$OUT/$tag.sock"
  export LOCUS_SOCK_PATH
  export LOCUS_AUDIT="$OUT/$tag.audit.ndjson"
  : > "$LOCUS_AUDIT"
  node "$SPIKE_DIR/proxy/locusd.mjs" > "$OUT/$tag.locusd.log" 2>&1 &
  LOCUSD_PID=$!
  for _ in $(seq 1 50); do
    [ -S "$LOCUS_SOCK_PATH" ] && grep -q '"ready":true' "$OUT/$tag.locusd.log" 2>/dev/null && return 0
    sleep 0.1
  done
  echo "FAIL: locusd did not become ready"; cat "$OUT/$tag.locusd.log"; return 1
}

stop_locusd() {
  [ -n "$LOCUSD_PID" ] && kill "$LOCUSD_PID" 2>/dev/null || true
  LOCUSD_PID=""
}

# Report whether the real credential is readable anywhere the container can see.
# Prints classes and counts, never values.
#
# Two scans, because they answer different questions:
#   targeted  the places a credential actually lands — env, PID environs, the
#             process table, the harness config dir, the workspace, the temp dirs
#   full      every readable text file in the container, as the backstop
#
# The needle is fed on STDIN and read into a NON-EXPORTED shell variable.
# Passing it with `docker exec -e` puts the credential into the exec'd process's
# own environment, and the scan then finds its own instrument — which it did,
# on the first run of this script, and reported a clean container as leaking.
#
# The /proc walks SNAPSHOT to a temp file and grep the file afterwards, for the
# same class of reason: `... | grep -F -- "$NEEDLE"` puts the needle into grep's
# OWN argv, which /proc/<grep-pid>/cmdline then reports as a hit. That false
# positive reported both clean candidates as leaking. Snapshot first, match
# second, and the instrument is no longer in its own sample.
#
# Reading one file per iteration matters too: cmdline entries carry no trailing
# newline, so `cat /proc/*/cmdline` collapses every process onto one line and a
# line count stops meaning anything.
_scan() {   # _scan <container> <shell-snippet-using-$NEEDLE>
  printf '%s' "$LOCUS_SPIKE_REAL_KEY" \
    | docker exec -i "$1" sh -c 'read -r NEEDLE
'"$2" 2>/dev/null | head -1 | tr -d ' \r'
}

assert_no_real_credential() {
  local cname="$1" label="$2" rc=0 n

  n="$(_scan "$cname" 'env | grep -Fc -- "$NEEDLE" || true')"
  echo "  $label env                  : ${n:-0} match(es)"
  [ "${n:-0}" = 0 ] || rc=1

  n="$(_scan "$cname" 'snap=$(mktemp)
        for f in /proc/[0-9]*/environ; do tr "\0" "\n" < "$f" 2>/dev/null; done > "$snap"
        grep -Fc -- "$NEEDLE" "$snap" || true
        rm -f "$snap"')"
  echo "  $label /proc/*/environ      : ${n:-0} match(es)"
  [ "${n:-0}" = 0 ] || rc=1

  n="$(_scan "$cname" 'snap=$(mktemp)
        for f in /proc/[0-9]*/cmdline; do tr "\0" " " < "$f" 2>/dev/null; echo; done > "$snap"
        grep -Fc -- "$NEEDLE" "$snap" || true
        rm -f "$snap"')"
  echo "  $label process cmdlines     : ${n:-0} match(es)"
  [ "${n:-0}" = 0 ] || rc=1

  n="$(_scan "$cname" 'grep -rlIF -- "$NEEDLE" /locus /workspace /root /home /tmp /var/tmp 2>/dev/null | wc -l')"
  echo "  $label config/workspace/tmp : ${n:-0} file(s)"
  [ "${n:-0}" = 0 ] || rc=1

  n="$(_scan "$cname" 'grep -rlIF -- "$NEEDLE" / --exclude-dir=proc --exclude-dir=sys --exclude-dir=dev 2>/dev/null | wc -l')"
  echo "  $label whole filesystem     : ${n:-0} file(s)"
  [ "${n:-0}" = 0 ] || rc=1

  return $rc
}
