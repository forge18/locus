#!/usr/bin/env bash
# Screens go through an accessor. The indirection is the whole reconciliation plan:
# at M1 each accessor's body becomes an invoke() and no screen changes. A screen
# reaching into src/fixtures/ directly is a screen that will have to be rewritten.
set -euo pipefail
cd "$(dirname "$0")/.."

# src/data/ is the seam and is supposed to import fixtures. Everything else is not.
hits=$(grep -rnE "from ['\"][^'\"]*fixtures/" src \
        --include='*.ts' --include='*.tsx' \
        --exclude-dir=fixtures \
        --exclude-dir=data || true)

if [ -n "$hits" ]; then
  echo "these import a fixture directly instead of going through src/data/:"
  echo "$hits"
  exit 1
fi
echo "check-no-direct-fixture-import: only src/data/ reads src/fixtures/"
