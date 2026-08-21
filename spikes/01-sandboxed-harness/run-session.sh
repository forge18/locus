#!/usr/bin/env bash
# Tasks 9-11 — a real harness session inside the container, captured and
# normalized to the canonical vocabulary.
#
# The container is configured as CANDIDATE A: it holds a sentinel and a base URL,
# and the host daemon swaps in the real credential. So this script proves two
# things at once — that the events arrive, and that they arrive through the
# credential boundary rather than around it.
#
# Usage: run-session.sh [hooks|stream-json]
#   hooks        the reference capture path; writes out/claude.events.json
#   stream-json  a second capture SOURCE through the same binary; writes
#                out/claude.stream.events.json
#
# Requires a real provider credential in LOCUS_SPIKE_REAL_KEY. The synthetic key
# lib.sh generates answers the exposure question but cannot talk to a model, and
# a spike that reported captured events from a mock would be answering a
# question nobody asked.
set -euo pipefail
cd "$(dirname "$0")"
. ./lib.sh

MODE="${1:-hooks}"
TAG="session-$MODE"
CNAME="locus-spike-$TAG-$$"
NONCE="nonce-$(openssl rand -hex 8)"
RUN_ID="$TAG-$$"

STORE="${LOCUS_CRED_STORE:-$HOME/.local/state/locus-spike/credentials.json}"
if [ "$LOCUS_KEY_ORIGIN" != "operator-supplied" ] && [ ! -s "$STORE" ]; then
  cat >&2 <<'MSG'
run-session.sh needs a real provider credential. There are two ways in and the
spike supports both:

  ./set-credential.sh api-key     paste an Anthropic API key
  ./set-credential.sh sign-in     hand over an existing Claude Code OAuth token

Either lands in a host-side store outside this repo, mode 0600. Neither is ever
passed to the container, written into the spike tree, or logged — scan-secrets.sh
and try-cred-kinds.sh are the checks on that claim, not this sentence.
MSG
  exit 3
fi
# The synthetic key lib.sh generates would let the run start and then fail at
# the model. Clear it so the daemon reads the operator's store instead.
[ "$LOCUS_KEY_ORIGIN" = "operator-supplied" ] || unset LOCUS_SPIKE_REAL_KEY

export LOCUS_UPSTREAM="${LOCUS_UPSTREAM:-https://api.anthropic.com}"
export LOCUS_RUN_NONCE="$NONCE"
export LOCUS_EGRESS_TIER=model

cleanup() { docker rm -f "$CNAME" >/dev/null 2>&1 || true; stop_locusd; }
trap cleanup EXIT

start_locusd "$TAG"
echo "live session — mode=$MODE  upstream=$LOCUS_UPSTREAM  tier=$LOCUS_EGRESS_TIER"

PROMPT='Read the file /workspace/README.md. Then create /workspace/NOTES.md containing exactly one line: "spike 1 ok". Do not explain, just do it.'

# The materialized config is mounted READ-ONLY, per PLAN.md's mount table. The
# entrypoint copies it to a writable /locus/config because Claude Code's config
# home is also where it writes transcripts — see the note in locus-entrypoint.
docker run -d --name "$CNAME" \
  --add-host=host.docker.internal:host-gateway \
  -v "$PWD/config:/locus/config-ro:ro" \
  -e LOCUS_RPC_ADDR="host.docker.internal:${LOCUS_RPC_PORT:-43802}" \
  -e LOCUS_RUN_NONCE="$NONCE" \
  -e LOCUS_RUN_ID="$RUN_ID" \
  -e ANTHROPIC_BASE_URL="http://host.docker.internal:$LOCUS_PROXY_PORT" \
  -e ANTHROPIC_API_KEY="$LOCUS_SENTINEL" \
  locus/base-claude sleep 900 >/dev/null
sleep 1

echo "--- running the harness ---"
if [ "$MODE" = "stream-json" ]; then
  docker exec "$CNAME" sh -c "
    cd /workspace
    claude -p '$PROMPT' --permission-mode bypassPermissions \
      --output-format stream-json --verbose --include-partial-messages=false \
      > /locus/stdout.ndjson 2>/locus/stderr.log
    echo \"harness exit: \$?\"" || echo "harness exited non-zero (captured below)"
  docker exec "$CNAME" sh -c 'cat /locus/stderr.log' | tail -20
  docker cp "$CNAME:/locus/stdout.ndjson" "$OUT/claude.stdout.ndjson" 2>/dev/null || : > "$OUT/claude.stdout.ndjson"
  node normalize.mjs stream-json "$RUN_ID" "$OUT/claude.stream.events.json" "$OUT/claude.stdout.ndjson"
else
  docker exec "$CNAME" sh -c "
    cd /workspace
    claude -p '$PROMPT' --permission-mode bypassPermissions > /locus/stdout.txt 2>/locus/stderr.log
    echo \"harness exit: \$?\"" || echo "harness exited non-zero (captured below)"
  docker exec "$CNAME" sh -c 'cat /locus/stderr.log' | tail -20
  docker cp "$CNAME:/locus/events.ndjson" "$OUT/claude.hooks.ndjson" 2>/dev/null || : > "$OUT/claude.hooks.ndjson"
  rm -rf "$OUT/transcripts"; mkdir -p "$OUT/transcripts"
  docker exec "$CNAME" sh -c 'find /locus/config/projects -name "*.jsonl" 2>/dev/null | head -20' \
    | while read -r f; do [ -n "$f" ] && docker cp "$CNAME:$f" "$OUT/transcripts/$(basename "$f")" 2>/dev/null || true; done
  node normalize.mjs hooks-claude "$RUN_ID" "$OUT/claude.events.json" \
    "$OUT/claude.hooks.ndjson" "$OUT"/transcripts/*.jsonl
fi

echo "--- the credential boundary, during a real session ---"
if assert_no_real_credential "$CNAME" " "; then
  echo "  CLEAN: the real credential was not in the container at any point this run"
else
  echo "  PRESENT: the real credential leaked into the container" >&2
  exit 1
fi

echo "--- audit ---"
grep '"verb":"egress"' "$OUT/$TAG.audit.ndjson" | jq -c '{status, injected, credential_presented}' | sort | uniq -c
leak="$(grep -Fc -- "$LOCUS_SPIKE_REAL_KEY" "$OUT/$TAG.audit.ndjson" || true)"
[ "${leak:-0}" = 0 ] || { echo "FAIL: the audit log contains the credential" >&2; exit 1; }
echo "  audit log holds 0 occurrences of the credential"
