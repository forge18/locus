#!/usr/bin/env bash
# CANDIDATE A — host credential proxy.
#
# The container is given a SENTINEL that looks like a key and is worth nothing.
# ANTHROPIC_BASE_URL points at the host daemon, which swaps the sentinel for the
# real credential on the way out. The real credential never enters the container
# and never crosses the container network.
#
# What this must show, to count:
#   A1  a call made with the sentinel through the proxy SUCCEEDS
#   A2  the same sentinel presented directly to the upstream FAILS
#   A3  the real credential is absent from the container's env and filesystem
#   A4  the container cannot read the real credential back out of the proxy
#   A5  egress policy and the audit row live at this same chokepoint
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=proxy
CNAME="locus-spike-$TAG-$$"
NONCE="nonce-$(openssl rand -hex 8)"
RESULT="$OUT/$TAG.result.json"

cleanup() { docker rm -f "$CNAME" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

start_locusd "$TAG"
echo "candidate A — host credential proxy   (key origin: $LOCUS_KEY_ORIGIN)"

# The container's whole credential configuration. Note what is NOT here.
docker run -d --name "$CNAME" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_NONCE="$NONCE" \
  -e LOCUS_RUN_ID="$TAG-$$" \
  -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_PROXY_PORT" \
  -e ANTHROPIC_API_KEY="$LOCUS_SENTINEL" \
  locus/base-claude sleep 600 >/dev/null

sleep 1
pass=0; fail=0
chk() { if [ "$1" = "$2" ]; then echo "  PASS $3"; pass=$((pass+1)); else echo "  FAIL $3 (got '$1', want '$2')"; fail=$((fail+1)); fi; }

# A1 — through the proxy, with the sentinel.
a1="$(docker exec "$CNAME" sh -c '
  curl -s -o /dev/null -w "%{http_code}" \
    -H "x-api-key: $ANTHROPIC_API_KEY" -H "content-type: application/json" \
    -d "{\"model\":\"probe\",\"messages\":[]}" "$ANTHROPIC_BASE_URL/v1/messages"')"
chk "$a1" "200" "A1 sentinel through the proxy is accepted upstream"

# A2 — same sentinel, straight at the upstream, proxy bypassed.
a2="$(docker exec "$CNAME" sh -c "
  curl -s -o /dev/null -w '%{http_code}' \
    -H \"x-api-key: \$ANTHROPIC_API_KEY\" -H 'content-type: application/json' \
    -d '{}' http://host.docker.internal:$LOCUS_MOCK_PORT/v1/messages")"
chk "$a2" "401" "A2 sentinel presented directly to upstream is rejected"

# A3 — the exposure question.
echo "  --- what the container holds ---"
if assert_no_real_credential "$CNAME" "A3"; then
  echo "  PASS A3 real credential absent from container env and filesystem"; pass=$((pass+1))
else
  echo "  FAIL A3 real credential is READABLE inside the container"; fail=$((fail+1))
fi

# A4 — the proxy must not be a credential oracle: nothing it returns contains
# the real key, and no verb hands it back.
a4="$(docker exec "$CNAME" sh -c '
  { curl -s -H "x-api-key: $ANTHROPIC_API_KEY" -d "{}" "$ANTHROPIC_BASE_URL/v1/messages";
    locus creds get 2>/dev/null; } | grep -Fc -- "$1" || true' _ "$LOCUS_SPIKE_REAL_KEY")"
chk "$a4" "0" "A4 proxy responses never echo the real credential"

# A5 — the same chokepoint carries policy and audit.
stop_locusd
LOCUS_EGRESS_TIER=none start_locusd "$TAG-tier-none"
a5="$(docker exec "$CNAME" sh -c '
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $ANTHROPIC_API_KEY" \
    -d "{}" "$ANTHROPIC_BASE_URL/v1/messages"')"
chk "$a5" "403" "A5 egress tier 'none' refuses at the injection chokepoint"
rows="$(grep -c '"verb":"egress"' "$OUT/$TAG.audit.ndjson" 2>/dev/null || echo 0)"
if [ "$rows" -ge 2 ]; then echo "  PASS A5 audit rows written per outbound call ($rows)"; pass=$((pass+1));
else echo "  FAIL A5 no audit rows"; fail=$((fail+1)); fi
leak="$(grep -Fc -- "$LOCUS_SPIKE_REAL_KEY" "$OUT/$TAG.audit.ndjson" 2>/dev/null || true)"
chk "${leak:-0}" "0" "A5 audit log records the credential CLASS, never its value"

cat > "$RESULT" <<JSON
{"candidate":"A","name":"host credential proxy","pass":$pass,"fail":$fail,
 "container_holds":"a sentinel string and a base URL",
 "exposure_window":"none — the real credential never enters the container",
 "revocable_mid_run":true,
 "setup_cost":"one host daemon per machine; nothing per project",
 "carries_egress_policy":true,"carries_audit":true,
 "key_origin":"$LOCUS_KEY_ORIGIN"}
JSON
echo "candidate A: $pass passed, $fail failed  -> $RESULT"
exit "$fail"
