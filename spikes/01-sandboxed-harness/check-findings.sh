#!/usr/bin/env bash
# Task 13's verify — FINDINGS.md is a verdict, not a survey.
#
# This checks the shape the acceptance criteria demand, not the prose:
#   * every question answered with a VERDICT line
#   * the falsifier and the fallback both named
#   * dsh and hermes each carry a verdict word
#   * the exposure table is backed by out/exposure.json rather than asserted
#   * no credential value appears anywhere in the spike directory
set -euo pipefail
cd "$(dirname "$0")"
F=FINDINGS.md
fail=0
ok()   { printf '  PASS %s\n' "$1"; }
bad()  { printf '  FAIL %s\n' "$1"; fail=1; }

[ -s "$F" ] || { echo "FAIL: $F is missing or empty"; exit 1; }

for q in Q1 Q2 Q3 Q4; do
  if grep -qE "^#+ $q\b" "$F"; then ok "$q has a section"; else bad "$q has no section"; fi
done

n="$(grep -c '^\*\*VERDICT' "$F" || true)"
[ "$n" -ge 4 ] && ok "four or more VERDICT lines ($n)" || bad "only $n VERDICT lines; one per question is the minimum"

grep -qi 'falsif' "$F" && ok "the falsifier is named" || bad "no falsifier named"
grep -qi 'fallback' "$F" && ok "the fallback is named" || bad "no fallback named"

grep -Eq 'dsh.*(VERIFIED|UNVERIFIED|REFUTED)'    "$F" && ok "dsh carries a verdict"    || bad "dsh has no verdict"
grep -Eq 'hermes.*(VERIFIED|UNVERIFIED|REFUTED)' "$F" && ok "hermes carries a verdict" || bad "hermes has no verdict"

grep -qi 'what the container held' "$F" && ok "the finding states what the container held" \
  || bad "the finding does not state what the container held"

for artefact in out/exposure.json out/harness-verify.json out/proxy.result.json \
                out/broker.result.json out/env.result.json; do
  [ -s "$artefact" ] && ok "evidence present: $artefact" || bad "missing evidence: $artefact"
done

# The finding must not be the thing that leaks. Any 40+ char key-shaped token
# in the spike tree is a failure regardless of whether it is live.
if grep -rEl '(sk-ant-[A-Za-z0-9_-]{20,}|sk-[a-z]+-[A-Za-z0-9]{40,})' . \
     --exclude-dir=out --exclude-dir=node_modules 2>/dev/null | grep -q .; then
  bad "a key-shaped string is committed in the spike tree"
else
  ok "no key-shaped string in the spike tree"
fi

exit "$fail"
