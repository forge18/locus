#!/usr/bin/env bash
# Components consume semantic theme roles. The compatibility aliases are compatibility-only
# declarations in tokens.css while the migration completes.
set -euo pipefail
cd "$(dirname "$0")/.."

legacy='(bg|bg-deep|sf|sf2|sf3|blue|blue-lit|ac|ac2|tx|mu|mu2|line|line2|ok|bad)'
hits=$(rg -n --glob '*.{css,ts,tsx}' "var\\(--${legacy}\\)" src | grep -v '^src/styles/tokens.css:' || true)
if [ -n "$hits" ]; then
  echo "legacy theme aliases outside src/styles/tokens.css:"
  echo "$hits"
  exit 1
fi

theme_selectors=$(rg -n --glob '*.{css,ts,tsx}' 'data-theme' src | grep -v '^src/styles/tokens.css:' || true)
if [ -n "$theme_selectors" ]; then
  echo "theme selectors outside src/styles/tokens.css:"
  echo "$theme_selectors"
  exit 1
fi

echo "check-theme-token-boundary: components use semantic theme roles"
