#!/usr/bin/env bash
# Q1, second half — the mechanism must carry BOTH ways a person gets in.
#
# "Offer a way for users to add an API key or sign in" is not two features; it
# is one chokepoint with two inputs. Most people running Claude Code are on a
# subscription and have no API key at all; anyone driving it from CI has a key
# and no browser. A credential design that serves one of them serves half.
#
# What this must show, to count:
#   K1  an api_key credential injects as x-api-key and is accepted
#   K2  an oauth credential injects as authorization: Bearer, with the oauth beta
#       header, and is accepted — even though the CONTAINER presented x-api-key
#   K3  an expired oauth access token is refreshed on the host before injection,
#       with no error reaching the container
#   K4  neither the access token nor the refresh token is ever in the container
#   K5  the refresh token never leaves the host process, even to the upstream
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=cred-kinds
CNAME="locus-spike-$TAG-$$"
cleanup() { docker rm -f "$CNAME" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

pass=0; fail=0
chk() { if [ "$1" = "$2" ]; then echo "  PASS $3"; pass=$((pass+1)); else echo "  FAIL $3 (got '$1', want '$2')"; fail=$((fail+1)); fi; }

start_container() {
  docker rm -f "$CNAME" >/dev/null 2>&1 || true
  docker run -d --name "$CNAME" \
    --add-host=host.docker.internal:host-gateway \
    -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_PROXY_PORT" \
    -e ANTHROPIC_API_KEY="$LOCUS_SENTINEL" \
    locus/base-claude sleep 300 >/dev/null
  sleep 1
}

# The container's configuration is IDENTICAL for both kinds. That is the point:
# which credential the host holds is not the container's business.
call() { docker exec "$CNAME" sh -c '
  curl -s -o /dev/null -w "%{http_code}" -H "x-api-key: $ANTHROPIC_API_KEY" \
       -H "content-type: application/json" -d "{}" "$ANTHROPIC_BASE_URL/v1/messages"'; }

echo "=== kind: api_key ==="
export LOCUS_SPIKE_CRED_KIND=api_key
start_locusd "$TAG-api"
start_container
chk "$(call)" "200" "K1 api_key injects as x-api-key and is accepted"
saw="$(jq -r 'select(.verb=="upstream") | "\(.header)/\(.expected_header)"' "$OUT/$TAG-api.audit.ndjson" | tail -1)"
chk "$saw" "x-api-key/x-api-key" "K1 upstream saw the x-api-key header"
stop_locusd

echo "=== kind: oauth ==="
ACCESS="oauth-access-$(openssl rand -hex 16)"
REFRESH="oauth-refresh-$(openssl rand -hex 16)"
export LOCUS_CRED_STORE="$OUT/$TAG.store.json"
umask 077
jq -n --arg a "$ACCESS" --arg r "$REFRESH" --arg u "http://127.0.0.1:${LOCUS_MOCK_PORT}/oauth/token" \
  '{kind:"oauth", access_token:$a, refresh_token:$r, refresh_url:$u,
    expires_at: 9999999999999}' > "$LOCUS_CRED_STORE"
unset LOCUS_SPIKE_REAL_KEY LOCUS_SPIKE_CRED_KIND
LOCUS_SPIKE_REAL_KEY="$ACCESS"   # for the scanner's needle only; not exported to locusd
start_locusd "$TAG-oauth"
grep -q '"credential_kind":"oauth"' "$OUT/$TAG-oauth.locusd.log" \
  && { echo "  PASS K2 the daemon loaded the oauth credential from the host store"; pass=$((pass+1)); } \
  || { echo "  FAIL K2 daemon did not load an oauth credential"; fail=$((fail+1)); }
start_container
chk "$(call)" "200" "K2 oauth injects as Bearer and is accepted, though the container sent x-api-key"
saw="$(jq -r 'select(.verb=="upstream") | "\(.header)/\(.expected_header)"' "$OUT/$TAG-oauth.audit.ndjson" | tail -1)"
chk "$saw" "authorization/authorization" "K2 upstream saw the authorization header, not x-api-key"

echo "=== kind: oauth, expired ==="
stop_locusd
jq '.expires_at = 1' "$LOCUS_CRED_STORE" > "$LOCUS_CRED_STORE.tmp" && mv "$LOCUS_CRED_STORE.tmp" "$LOCUS_CRED_STORE"
start_locusd "$TAG-refresh"
chk "$(call)" "200" "K3 an expired access token is refreshed on the host; the container sees a 200"
refreshed="$(grep -c '"action":"refresh"' "$OUT/$TAG-refresh.audit.ndjson" || true)"
[ "${refreshed:-0}" -ge 1 ] && { echo "  PASS K3 the refresh happened ($refreshed)"; pass=$((pass+1)); } \
                           || { echo "  FAIL K3 no refresh was recorded"; fail=$((fail+1)); }

echo "=== what the container holds ==="
export LOCUS_SPIKE_REAL_KEY="$ACCESS"
if assert_no_real_credential "$CNAME" "K4-access "; then
  echo "  PASS K4 the access token is not in the container"; pass=$((pass+1))
else
  echo "  FAIL K4 the access token leaked"; fail=$((fail+1))
fi
export LOCUS_SPIKE_REAL_KEY="$REFRESH"
if assert_no_real_credential "$CNAME" "K5-refresh"; then
  echo "  PASS K5 the refresh token is not in the container"; pass=$((pass+1))
else
  echo "  FAIL K5 the refresh token leaked"; fail=$((fail+1))
fi
n="$(grep -Fc -- "$REFRESH" "$OUT/$TAG-refresh.audit.ndjson" || true)"
chk "${n:-0}" "0" "K5 the refresh token is not in the audit log"

cat > "$OUT/cred-kinds.result.json" <<JSON
{"pass":$pass,"fail":$fail,
 "kinds_proven":["api_key","oauth","oauth-expired-then-refreshed"],
 "container_config_identical_across_kinds":true,
 "note":"the container presented x-api-key in every case; the header on the wire is chosen by the credential the HOST holds"}
JSON
echo "cred kinds: $pass passed, $fail failed"
exit "$fail"
