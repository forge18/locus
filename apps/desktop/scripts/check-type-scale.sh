#!/usr/bin/env bash
# Type sizes come from the scale, and the scale has a floor.
#
# The handoff drew 9-9.5px for two thirds of the app's text. Sizes are tokens now,
# so the whole scale moves in one edit — and a raw px font-size anywhere else is
# how it would quietly stop moving.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

raw=$(grep -rn 'font-size: *[0-9.]\+px' src \
        --include='*.css' --include='*.ts' --include='*.tsx' \
        --exclude-dir=assets \
        | grep -v '^src/styles/tokens.css:' || true)
if [ -n "$raw" ]; then
  echo "raw px font sizes outside the scale:"
  echo "$raw"
  fail=1
fi

# The floor: nothing in the scale is smaller than 11px.
below=$(grep -oE '\-\-t-[a-z-]+: *([0-9.]+)px' src/styles/tokens.css \
        | awk -F': *' '{ gsub("px","",$2); if ($2+0 < 11) print $0 }' || true)
if [ -n "$below" ]; then
  echo "type tokens below the 11px floor:"
  echo "$below"
  fail=1
fi

# And body text is at least 14px, which is the size the rows are actually read at.
body=$(grep -oE '\-\-t-body: *([0-9.]+)px' src/styles/tokens.css | grep -oE '[0-9.]+')
if [ "$(echo "$body < 14" | bc)" = "1" ]; then
  echo "--t-body is ${body}px; rows are read at this size and it must be at least 14px"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "check-type-scale: every size comes from the scale; floor 11px, body ${body}px"
fi
exit "$fail"
