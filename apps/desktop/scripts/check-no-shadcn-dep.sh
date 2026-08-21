#!/usr/bin/env bash
# shadcn-solid is copied in, not depended on. Kobalte is the only runtime dep the
# component library adds — that is the point of owning the source.
set -euo pipefail
cd "$(dirname "$0")/.."

if grep -qE '"[^"]*shadcn[^"]*"\s*:' package.json; then
  echo "a shadcn package is listed as a dependency; the components are copied in, not installed"
  grep -nE '"[^"]*shadcn[^"]*"\s*:' package.json
  exit 1
fi

grep -q '"@kobalte/core"' package.json || { echo "@kobalte/core is missing"; exit 1; }

# It has to be a runtime dependency, not a devDependency — the app renders with it.
node -e '
  const p = require("./package.json");
  if (!p.dependencies?.["@kobalte/core"]) {
    console.error("@kobalte/core must be a runtime dependency");
    process.exit(1);
  }
' || exit 1

echo "check-no-shadcn-dep: Kobalte is the only component dependency; shadcn is source"
