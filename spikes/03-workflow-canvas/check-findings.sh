#!/usr/bin/env bash
# Task 13's verify. The acceptance criterion is a BINARY RECOMMENDATION, not a
# comparison, so that is what this checks for — plus the shared-renderer verdict
# the spec leaves open, and the evidence behind both.
set -euo pipefail
cd "$(dirname "$0")"
F=FINDINGS.md
fail=0
ok()  { printf '  PASS %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

[ -s "$F" ] || { echo "FAIL: $F is missing or empty"; exit 1; }

for q in Q1 Q2 Q3 Q4; do
  grep -qE "^#+ $q\b" "$F" && ok "$q has a section" || bad "$q has no section"
done

# Binary means one of two words, stated once, not "on balance".
if grep -qE '^\*\*VERDICT: (solid-flow|the dagre fallback)\b' "$F"; then
  ok "the recommendation is binary"
else
  bad "no binary recommendation — expected 'VERDICT: solid-flow' or 'VERDICT: the dagre fallback'"
fi

grep -qiE 'wiki.*(share|shared).*renderer|renderer.*wiki' "$F" \
  && ok "the wiki shared-renderer question is answered" \
  || bad "the wiki shared-renderer question is unanswered"
grep -qi 'fallback' "$F" && ok "the fallback's cost is stated" || bad "the fallback is not mentioned"

[ -s screenshots/canvas.png ] && ok "screenshots/canvas.png exists" || bad "no canvas screenshot"

# The verdict has to rest on tests that actually run.
if pnpm test >/dev/null 2>&1; then ok "the whole suite passes"; else bad "the suite does not pass"; fi
if pnpm build >/dev/null 2>&1; then ok "the spike builds"; else bad "the spike does not build"; fi

exit "$fail"
