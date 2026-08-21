#!/usr/bin/env bash
# Task 11 — the second harness, on a second capture source.
#
# hermes in ACP mode, through the SAME credential boundary as claude: the
# container holds a sentinel and a base URL, and the host swaps in the real
# credential. If the boundary only worked for one harness's auth shape it would
# not be a boundary, it would be a coincidence.
#
# The ACP client runs on the HOST (PLAN.md §ACP puts it in the core), driving
# `hermes acp` inside the container over docker exec's stdio.
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

TAG=second
CNAME="locus-spike-$TAG-$$"
NONCE="nonce-$(openssl rand -hex 8)"
RUN_ID="$TAG-$$"

STORE="${LOCUS_CRED_STORE:-$HOME/.local/state/locus-spike/credentials.json}"
if [ "$LOCUS_KEY_ORIGIN" != "operator-supplied" ] && [ ! -s "$STORE" ]; then
  echo "run-second.sh needs a real provider credential — see ./set-credential.sh" >&2
  exit 3
fi
[ "$LOCUS_KEY_ORIGIN" = "operator-supplied" ] || unset LOCUS_SPIKE_REAL_KEY

export LOCUS_UPSTREAM="${LOCUS_UPSTREAM:-https://api.anthropic.com}"
export LOCUS_RUN_NONCE="$NONCE"
export LOCUS_EGRESS_TIER=model
MODEL="${LOCUS_SECOND_MODEL:-anthropic/claude-sonnet-4-5}"

cleanup() { docker rm -f "$CNAME" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

start_locusd "$TAG"
echo "second harness — hermes (acp)  model=$MODEL  upstream=$LOCUS_UPSTREAM"

# Same credential configuration as candidate A. Several base-URL spellings are
# set because which one hermes honours is exactly what is unknown; the finding
# records which took effect rather than guessing.
docker run -d --name "$CNAME" \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_NONCE="$NONCE" \
  -e LOCUS_RUN_ID="$RUN_ID" \
  -e ANTHROPIC_API_KEY="$LOCUS_SENTINEL" \
  -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_PROXY_PORT" \
  -e ANTHROPIC_API_BASE="http://host.docker.internal:$LOCUS_PROXY_PORT" \
  -e HERMES_ACCEPT_HOOKS=1 \
  locus/base-hermes sleep 900 >/dev/null
sleep 2

docker exec "$CNAME" sh -c "hermes config set default_model '$MODEL' >/dev/null 2>&1 || true"

PROMPT='Read /workspace/README.md and reply with its first line only. Do not explain.'
set +e
node acp-client.mjs "$CNAME" "$OUT/hermes.acp.ndjson" "$PROMPT" hermes acp --accept-hooks
ACP_RC=$?
set -e

echo "--- capture ---"
wc -l < "$OUT/hermes.acp.ndjson" | xargs echo "raw ACP messages:"
node normalize.mjs acp "$RUN_ID" "$OUT/second.events.json" "$OUT/hermes.acp.ndjson"

echo "--- the credential boundary, second harness ---"
if assert_no_real_credential "$CNAME" " "; then
  echo "  CLEAN: the credential was not in the container at any point this run"
else
  echo "  PRESENT: the credential leaked into the container" >&2; exit 1
fi

echo "--- audit: did the harness actually go through the proxy? ---"
jq -c 'select(.verb=="egress") | {status, injected, credential_presented}' "$OUT/$TAG.audit.ndjson" \
  | sort | uniq -c || true
if ! grep -q '"injected":true' "$OUT/$TAG.audit.ndjson"; then
  echo "  NOTE: no injected call was recorded — hermes did not honour the base URL." >&2
  echo "  That is a finding, not a failure of the boundary: record which env var it ignores." >&2
fi
exit "$ACP_RC"
