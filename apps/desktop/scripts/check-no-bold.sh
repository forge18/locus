#!/usr/bin/env bash
# 500 is the only emphasis weight for text.
#
# One documented exception: the inbox badge on the category rail is 700, because
# .specs/app-shell/spec.md rules it a pill rather than text — a 15px accent disc
# with a numeral inside it, where 500 reads as a smudge. It is named here rather
# than allowed by a looser pattern, so a second one cannot slip in beside it.
set -euo pipefail
cd "$(dirname "$0")/.."

ALLOWED_RULE='.rail-badge'

# Every font-weight declaration in hand-written source (src/assets/ is vendored),
# with the one allowed rule's block removed first.
hits=$(grep -rhoE "font-weight'?\"?\s*:\s*'?\"?[a-z0-9]+" src \
        --include='*.css' --include='*.ts' --include='*.tsx' \
        --exclude-dir=assets \
        --exclude-dir=shell || true)

# src/shell/ is scanned separately so the badge rule can be excised by name.
shell_css=$(awk -v allowed="$ALLOWED_RULE" '
  $0 ~ allowed" \\{" { skip = 1 }
  skip && /^\}/        { skip = 0; next }
  !skip                { print }
' src/shell/shell.css)
hits="$hits
$(echo "$shell_css" | grep -hoE "font-weight'?\"?\s*:\s*'?\"?[a-z0-9]+" || true)
$(grep -rhoE "font-weight'?\"?\s*:\s*'?\"?[a-z0-9]+" src/shell --include='*.tsx' --include='*.ts' || true)"

# The exception must actually be the badge, and there must be exactly one of it.
badge_count=$(grep -c 'font-weight: 700' src/shell/shell.css || true)
if [ "$badge_count" != "1" ]; then
  echo "expected exactly one 700 weight (the inbox badge); found $badge_count"
  exit 1
fi

bad=$(echo "$hits" | grep -vE ":\s*'?\"?(400|500|normal|inherit)$" | sed '/^$/d' || true)

if [ -n "$bad" ]; then
  echo "font weights above 500 are not part of this design:"
  echo "$bad"
  exit 1
fi
echo "check-no-bold: no weight above 500 in src/, except the named .rail-badge pill"
