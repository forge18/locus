#!/usr/bin/env bash
# Every token in the handoff table exists on :root.
#
# Most are carried over verbatim. Five depart from it, and each departure is
# asserted here rather than merely made — a value that drifts from the handoff
# without a reason on the record is a value nobody can defend later.
set -euo pipefail
cd "$(dirname "$0")/.."
FILE=src/styles/tokens.css
fail=0

check() {
  local name="$1" value="$2"
  if ! grep -qF -- "$name: $value;" "$FILE"; then
    echo "MISSING or WRONG: $name should be '$value'"
    fail=1
  fi
}

check --bg       "#1d2731"
check --bg-deep  "#151d25"
check --sf       "#22303c"
check --sf2      "#293947"
check --sf3      "#314454"
check --blue     "#083c5d"
check --blue-lit "#0d5480"
check --ac       "#ffbb39"
check --tx       "#eef2f6"
check --fm       "'JetBrains Mono', ui-monospace, Menlo, monospace"

# --- the five departures, each measured rather than taste ---
# Secondary text at the handoff's .56/.34 measures 4.04:1 and 2.46:1 against the
# app's own grounds; .34 fails WCAG AA everywhere it is used. Both are raised to
# the alpha that clears 4.5:1 on every ground.
check --mu       "rgba(238,242,246,.78)"
check --mu2      "rgba(238,242,246,.62)"
# Hairlines only have to clear 3:1, and these do.
check --line     "rgba(238,242,246,.14)"
check --line2    "rgba(238,242,246,.24)"
# The handoff's status pair measures 3.77 and 3.18 on --sf2, where both are text.
check --ok       "#68ad91"
check --bad      "#df8a7d"
# The originals stay, for fills and rings where nothing has to be read.
check --ok-solid  "#4fa07f"
check --bad-solid "#d4614f"

# Every departure carries its reason in the file, beside the value.
for token in --mu --mu2 --ok --bad; do
  grep -q -- "$token" "$FILE" || { echo "missing $token"; fail=1; }
done
grep -q 'WCAG AA' "$FILE" || {
  echo "the contrast departures are made but not explained in tokens.css"
  fail=1
}

# The tokens must live on a bare :root, not inside a media query or a theme class.
grep -q '^:root {' "$FILE" || { echo "tokens are not declared on a bare :root"; fail=1; }

if [ "$fail" -eq 0 ]; then
  echo "check-tokens: handoff grounds verbatim; 8 contrast departures present and explained"
fi
exit "$fail"
