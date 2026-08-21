#!/usr/bin/env bash
# CANDIDATE B — short-lived token minted per run over /run/locus.sock.
#
# The container starts with NO credential of any kind. When the harness needs to
# reach the model it asks the host daemon, which mints a token scoped to this run
# with a TTL. The token is not the provider's credential: the proxy is still the
# only thing that holds that, and it exchanges the token for it.
#
# What this must show, to count:
#   B1  at container start there is no credential anywhere in the container
#   B2  `locus creds get` mints a token, and the token works through the proxy
#   B3  the token is NOT the real credential, and presenting it upstream fails
#   B4  revoking mid-run kills it — the property candidate A does not have per-run
#   B5  the token expires on its own
#   B6  another container cannot use a token minted for this run's id
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=broker
CNAME="locus-spike-$TAG-$$"
NONCE="nonce-$(openssl rand -hex 8)"
RUN_ID="$TAG-$$"
RESULT="$OUT/$TAG.result.json"

OTHER="locus-spike-broker-other-$$"
cleanup() { docker rm -f "$CNAME" "$OTHER" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

export LOCUS_TOKEN_TTL_MS=4000
export LOCUS_RUN_NONCE="$NONCE"      # the daemon enforces it; see B6
start_locusd "$TAG"
echo "candidate B — per-run token over /run/locus.sock   (key origin: $LOCUS_KEY_ORIGIN, ttl ${LOCUS_TOKEN_TTL_MS}ms)"

# Note what is NOT passed: no ANTHROPIC_API_KEY, no sentinel, no base URL.
docker run -d --name "$CNAME" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_NONCE="$NONCE" \
  -e LOCUS_RUN_ID="$RUN_ID" \
  locus/base-claude sleep 600 >/dev/null
sleep 1

pass=0; fail=0
chk() { if [ "$1" = "$2" ]; then echo "  PASS $3"; pass=$((pass+1)); else echo "  FAIL $3 (got '$1', want '$2')"; fail=$((fail+1)); fi; }

# B1 — nothing at start.
b1="$(docker exec "$CNAME" sh -c 'env | grep -c "ANTHROPIC" || true')"
chk "$b1" "0" "B1 container starts with no provider credential in env"

# B2 — mint and use.
docker exec "$CNAME" sh -c 'locus creds get > /tmp/creds.json' 
b2="$(docker exec "$CNAME" sh -c '
  tok=$(jq -r .result.token /tmp/creds.json); url=$(jq -r .result.base_url /tmp/creds.json)
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $tok" -H "content-type: application/json" \
       -d "{}" "$url/v1/messages"')"
chk "$b2" "200" "B2 minted token is accepted through the proxy"

# B3 — the token is worthless off the chokepoint, and is not the real key.
b3a="$(docker exec "$CNAME" sh -c "
  tok=\$(jq -r .result.token /tmp/creds.json)
  curl -s -o /dev/null -w '%{http_code}' -H \"x-api-key: \$tok\" -d '{}' \
       http://host.docker.internal:$LOCUS_MOCK_PORT/v1/messages")"
chk "$b3a" "401" "B3 minted token presented directly to upstream is rejected"
echo "  --- what the container holds ---"
if assert_no_real_credential "$CNAME" "B3"; then
  echo "  PASS B3 real credential absent from container env and filesystem"; pass=$((pass+1))
else
  echo "  FAIL B3 real credential is READABLE inside the container"; fail=$((fail+1))
fi

# B4 — revocation mid-run. This is the property A does not have.
curl -sS --unix-socket "$LOCUS_SOCK_PATH" -H 'content-type: application/json' \
  -H "x-locus-run-nonce: " \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"creds.revoke\",\"params\":{\"run_id\":\"$RUN_ID\"}}" \
  http://locus/rpc >/dev/null 2>&1 || true
docker exec "$CNAME" sh -c 'locus creds revoke >/dev/null 2>&1' || true
b4="$(docker exec "$CNAME" sh -c '
  tok=$(jq -r .result.token /tmp/creds.json); url=$(jq -r .result.base_url /tmp/creds.json)
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $tok" -d "{}" "$url/v1/messages"')"
chk "$b4" "401" "B4 revoking mid-run kills the token immediately"

# B5 — expiry with no revocation.
docker exec "$CNAME" sh -c 'locus creds get > /tmp/creds2.json'
b5a="$(docker exec "$CNAME" sh -c '
  tok=$(jq -r .result.token /tmp/creds2.json); url=$(jq -r .result.base_url /tmp/creds2.json)
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $tok" -d "{}" "$url/v1/messages"')"
chk "$b5a" "200" "B5 a fresh token works"
docker exec "$CNAME" sh -c 'sleep 5'
b5b="$(docker exec "$CNAME" sh -c '
  tok=$(jq -r .result.token /tmp/creds2.json); url=$(jq -r .result.base_url /tmp/creds2.json)
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $tok" -d "{}" "$url/v1/messages"')"
chk "$b5b" "401" "B5 the same token is refused after its TTL elapses"

# B6 — the honest weakness. The RPC endpoint is TCP here (see locus-sockd), so
# every container on the host can reach it. The run nonce is what stops them.
docker run -d --name "$OTHER" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_ID="impostor" \
  locus/base-claude sleep 60 >/dev/null
sleep 1
b6="$(docker exec "$OTHER" sh -c 'locus creds get >/dev/null 2>&1 && echo minted || echo refused')"
chk "$b6" "refused" "B6 a container without this run's nonce cannot mint a token"

# B7 — and the hazard the nonce is covering, demonstrated rather than asserted.
# With enforcement off, the impostor mints freely. On Linux this case cannot
# arise because the socket is bind-mounted and unreachable from another
# container; on macOS the relay makes it reachable, so the nonce is load-bearing
# rather than defence in depth.
stop_locusd
unset LOCUS_RUN_NONCE
start_locusd "$TAG-nononce"
docker rm -f "$OTHER" >/dev/null 2>&1 || true
docker run -d --name "$OTHER" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_ID="impostor" \
  locus/base-claude sleep 60 >/dev/null
sleep 1
b7="$(docker exec "$OTHER" sh -c 'locus creds get >/dev/null 2>&1 && echo minted || echo refused')"
docker rm -f "$OTHER" >/dev/null 2>&1 || true
chk "$b7" "minted" "B7 with the nonce disabled, any container on the host mints — the relay's cost, shown"

cat > "$RESULT" <<JSON
{"candidate":"B","name":"per-run token over /run/locus.sock","pass":$pass,"fail":$fail,
 "container_holds":"nothing at start; a revocable per-run token once it asks",
 "exposure_window":"the token's TTL, and only after the harness asks for one",
 "revocable_mid_run":true,
 "setup_cost":"one host daemon per machine, plus a harness that can be pointed at a base URL AND told to fetch its own credential",
 "carries_egress_policy":true,"carries_audit":true,
 "key_origin":"$LOCUS_KEY_ORIGIN"}
JSON
echo "candidate B: $pass passed, $fail failed  -> $RESULT"
exit "$fail"
