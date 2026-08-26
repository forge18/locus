#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

palette="src/nav/LocatorPalette.tsx"
store="src/nav/store.ts"

rg -q 'navigateDesktop' "$palette"
rg -q 'resolve\(' "$store"
if rg -n 'parse\(' "$palette"; then
  echo "LocatorPalette must delegate locator parsing to navigation" >&2
  exit 1
fi
printf 'palette and navigation share one resolver\n'
