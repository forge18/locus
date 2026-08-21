#!/usr/bin/env bash
# The built app must not reference a host. Fonts and icons are vendored; a CDN
# reference here is a blank screen on a machine with no network.
#
# This proves the bundle asks for nothing remote. It does not launch the app with
# the interface down — that check belongs to a packaged build.
set -euo pipefail
cd "$(dirname "$0")/.."

pnpm build >/dev/null

# The SVG namespace URI is a name, not a fetch, and `locus://` is this app's own
# address space. Everything else is suspect.
hits=$(grep -rhoE '(https?:|locus:)?//[A-Za-z0-9._-]+' dist \
        --include='*.js' --include='*.css' --include='*.html' \
        | grep -vE '//(www\.)?w3\.org' \
        | grep -v '^locus://' \
        | sort -u || true)

if [ -n "$hits" ]; then
  echo "the built app references external hosts:"
  echo "$hits"
  exit 1
fi

# And the faces themselves shipped.
for f in inter-latin-400-normal inter-latin-500-normal \
         jetbrains-mono-latin-400-normal jetbrains-mono-latin-500-normal; do
  ls dist/assets/"$f"*.woff2 >/dev/null 2>&1 || { echo "missing vendored face: $f"; exit 1; }
done

# The icon sprite is inlined into the bundle, so a symbol id must appear in the JS.
grep -q 'id="ph-tray"' dist/assets/*.js || { echo "icon sprite is not in the bundle"; exit 1; }

echo "check-offline: no external host, fonts and sprite shipped"
