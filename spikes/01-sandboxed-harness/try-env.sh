#!/usr/bin/env bash
# CANDIDATE C — the real credential injected as an env var at container start.
#
# This is the honest baseline, not a straw man. It is what most of the field
# does, it needs no daemon, no proxy, and no harness cooperation beyond reading
# the variable the harness already reads. If A and B cannot beat it on exposure
# by more than they cost, the finding has to say so.
#
# What this measures:
#   C1  it works, with nothing on the host running
#   C2  the real credential IS readable inside the container — how, and where
#   C3  it cannot be revoked mid-run
#   C4  it is off the chokepoint: no egress policy, no audit row
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=env
CNAME="locus-spike-$TAG-$$"
RESULT="$OUT/$TAG.result.json"

cleanup() { docker rm -f "$CNAME" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

start_locusd "$TAG"     # only for the upstream; the container never talks to the proxy
echo "candidate C — env var at container start   (key origin: $LOCUS_KEY_ORIGIN)"

docker run -d --name "$CNAME" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RUN_ID="$TAG-$$" \
  -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_MOCK_PORT" \
  -e ANTHROPIC_API_KEY="$LOCUS_SPIKE_REAL_KEY" \
  locus/base-claude sleep 600 >/dev/null
sleep 1

pass=0; fail=0
chk() { if [ "$1" = "$2" ]; then echo "  PASS $3"; pass=$((pass+1)); else echo "  FAIL $3 (got '$1', want '$2')"; fail=$((fail+1)); fi; }

# C1 — it works, straight at the upstream.
c1="$(docker exec "$CNAME" sh -c '
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $ANTHROPIC_API_KEY" \
       -H "content-type: application/json" -d "{}" "$ANTHROPIC_BASE_URL/v1/messages"')"
chk "$c1" "200" "C1 the credential works with no host daemon in the path"

# C2 — the exposure, measured the same way as A and B so the numbers compare.
echo "  --- what the container holds ---"
if assert_no_real_credential "$CNAME" "C2"; then
  echo "  UNEXPECTED C2 the credential was not found — the scan is wrong, not the candidate"; fail=$((fail+1))
else
  echo "  EXPECTED  C2 the real credential is readable inside the container (this is the candidate's cost)"; pass=$((pass+1))
fi

# Where, specifically. Any process in the container can read PID 1's environ.
where="$(printf '%s' "$LOCUS_SPIKE_REAL_KEY" | docker exec -i "$CNAME" sh -c '
  read -r NEEDLE
  for f in /proc/1/environ /proc/self/environ; do
    tr "\0" "\n" < "$f" 2>/dev/null | grep -qF -- "$NEEDLE" && echo "$f"
  done
  exit 0' | tr '\n' ' ' || true)"
echo "  C2 readable at: ${where:-none}"

# C3 — revocation. There is no mechanism; the env var is fixed at container
# creation and rotating it means killing the run.
c3="$(docker exec "$CNAME" sh -c 'unset ANTHROPIC_API_KEY; cat /proc/1/environ | tr "\0" "\n" | grep -c "^ANTHROPIC_API_KEY=" || true')"
chk "$c3" "1" "C3 the credential survives being unset in a child shell — it lives on PID 1"

# C4 — nothing was audited, because nothing passed the chokepoint.
rows="$(grep -c '"verb":"egress"' "$OUT/$TAG.audit.ndjson" 2>/dev/null || true)"
chk "${rows:-0}" "0" "C4 no audit row exists — the call never crossed the chokepoint"

cat > "$RESULT" <<JSON
{"candidate":"C","name":"env var at container start","pass":$pass,"fail":$fail,
 "container_holds":"the real, long-lived provider credential",
 "exposure_window":"the entire life of the container, readable by every process in it",
 "revocable_mid_run":false,
 "setup_cost":"none — no daemon, no proxy, no harness cooperation",
 "carries_egress_policy":false,"carries_audit":false,
 "readable_at":"${where:-none}",
 "key_origin":"$LOCUS_KEY_ORIGIN"}
JSON
echo "candidate C: $pass passed, $fail failed  -> $RESULT"
exit "$fail"
