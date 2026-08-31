#!/usr/bin/env bash
# Production screens and accessors may not reach fixture data or fixture screens.
# Demo/test hosts are the only explicit exceptions.
set -euo pipefail
cd "$(dirname "$0")/.."

fixture_imports=$(grep -rnE "from ['\"](\\.{1,2}/)+fixtures/" src \
  --include='*.ts' --include='*.tsx' \
  --exclude-dir=fixtures \
  --exclude-dir=demo \
  --exclude-dir=test || true)
fixture_components=$(grep -rnE "WorkshopFixtureView|Memory[A-Za-z]+Fixture|Desktop_FIXTURE_ROUTES" src \
  --include='*.ts' --include='*.tsx' \
  --exclude-dir=fixtures \
  --exclude-dir=demo \
  --exclude-dir=test || true)

if [ -n "$fixture_imports" ] || [ -n "$fixture_components" ]; then
  echo "production source reached the demo fixture boundary:"
  [ -z "$fixture_imports" ] || echo "$fixture_imports"
  [ -z "$fixture_components" ] || echo "$fixture_components"
  exit 1
fi
echo "check-no-direct-fixture-import: production source has no fixture imports or fixture screens"
