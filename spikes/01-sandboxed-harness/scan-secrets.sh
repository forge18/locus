#!/usr/bin/env bash
# Task 8 — scan each candidate's container for a persisted credential.
#
# The three try-*.sh scripts each scan their own container. This runs the same
# scan across all three in one invocation so the numbers are comparable, and
# writes out/exposure.json, which FINDINGS.md's exposure table is generated from.
#
# Scanned per candidate: process environment, every PID's environ, every PID's
# cmdline, the harness config dir, the workspace, the temp dirs, and finally
# every readable text file in the container.
#
# No credential value is printed. Ever. Only counts and the paths that matched.
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=scan
NONCE="nonce-$(openssl rand -hex 8)"
CONTAINERS=()
cleanup() { [ ${#CONTAINERS[@]} -gt 0 ] && docker rm -f "${CONTAINERS[@]}" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

export LOCUS_RUN_NONCE="$NONCE"
start_locusd "$TAG"

run_candidate() {   # run_candidate <letter> <extra docker args...>
  local letter="$1"; shift
  local cname="locus-spike-scan-$letter-$$"
  CONTAINERS+=("$cname")
  docker run -d --name "$cname" \
    --add-host=host.docker.internal:host-gateway \
    -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
    -e LOCUS_RUN_NONCE="$NONCE" \
    -e LOCUS_RUN_ID="scan-$letter" \
    "$@" locus/base-claude sleep 300 >/dev/null
  echo "$cname"
}

echo "exposure scan — key origin: $LOCUS_KEY_ORIGIN"
echo

A="$(run_candidate A -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_PROXY_PORT" \
                      -e ANTHROPIC_API_KEY="$LOCUS_SENTINEL")"
B="$(run_candidate B)"
C="$(run_candidate C -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_MOCK_PORT" \
                      -e ANTHROPIC_API_KEY="$LOCUS_SPIKE_REAL_KEY")"
sleep 1

# B only holds anything once the harness asks, so make it ask — scanning B
# before its first `creds get` would flatter it.
docker exec "$B" sh -c 'locus creds get > /tmp/creds.json' >/dev/null

declare -a VERDICTS
for pair in "A:$A" "B:$B" "C:$C"; do
  letter="${pair%%:*}"; cname="${pair#*:}"
  echo "candidate $letter — $cname"
  if assert_no_real_credential "$cname" " "; then
    echo "  => CLEAN: the real credential is not present in this container"
    VERDICTS+=("$letter:clean")
  else
    echo "  => PRESENT: the real credential is readable in this container"
    VERDICTS+=("$letter:present")
  fi
  # What the container DOES hold, named rather than counted.
  holds="$(docker exec "$cname" sh -c '
    env | grep -E "^ANTHROPIC_(API_KEY|BASE_URL)=" | sed "s/=.*/=<redacted>/" | tr "\n" " "
    [ -f /tmp/creds.json ] && echo -n "run-token=<redacted> "
    true')"
  echo "  holds: ${holds:-<nothing>}"
  echo
done

{
  echo '{'
  echo '  "key_origin": "'"$LOCUS_KEY_ORIGIN"'",'
  echo '  "candidates": {'
  first=1
  for v in "${VERDICTS[@]}"; do
    [ $first -eq 1 ] || echo ','
    first=0
    printf '    "%s": {"real_credential": "%s"}' "${v%%:*}" "${v#*:}"
  done
  echo
  echo '  }'
  echo '}'
} > "$OUT/exposure.json"

echo "wrote $OUT/exposure.json"
cat "$OUT/exposure.json"

# The scan is only meaningful if it can find a credential that IS there.
# Candidate C is the positive control: if C reads clean, the instrument is broken.
if grep -q '"C": {"real_credential": "present"}' "$OUT/exposure.json"; then
  echo
  echo "positive control OK — the scan finds a credential when one is present"
  exit 0
fi
echo
echo "FAIL: positive control did not trip. The scan is not trustworthy." >&2
exit 1
