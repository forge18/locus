#!/usr/bin/env bash
# The Tauri + Solid scaffold is gone. Leaving a greet handler or a framework logo
# in the shipped app is how a demo ends up in a screenshot.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

hits=$(grep -rniE 'greetMsg|setGreetMsg|invoke\("greet"|Welcome to Tauri|vite\.svg|tauri\.svg|logo\.svg' \
        src index.html --include='*.tsx' --include='*.ts' --include='*.html' --include='*.css' || true)
if [ -n "$hits" ]; then
  echo "the scaffold is still here:"
  echo "$hits"
  fail=1
fi

for f in src/App.css public/vite.svg public/tauri.svg src/assets/logo.svg; do
  if [ -e "$f" ]; then echo "scaffold asset still present: $f"; fail=1; fi
done

# And the shell is what App renders now.
grep -q '<Shell' src/App.tsx || { echo "App.tsx does not render the shell"; fail=1; }

if [ "$fail" -eq 0 ]; then echo "check-scaffold-removed: the scaffold is gone and App renders the shell"; fi
exit "$fail"
