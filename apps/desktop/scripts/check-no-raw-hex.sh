#!/usr/bin/env bash
# No hardcoded color outside the token file and the fixtures. A screen that writes
# a hex is a screen that will not move when --ac moves.
#
# Takes an optional path to narrow the scan; defaults to all of src/.
set -euo pipefail
here="$PWD"
cd "$(dirname "$0")/.."

target="src"
if [ "$#" -gt 0 ]; then
  if [ -e "$here/$1" ]; then target="$here/$1"
  elif [ -e "$1" ]; then target="$1"
  else echo "no such path: $1"; exit 1
  fi
fi

hits=$(grep -rnE '#[0-9a-fA-F]{3}([0-9a-fA-F]{3}([0-9a-fA-F]{2})?)?\b' "$target" \
        --include='*.css' --include='*.ts' --include='*.tsx' \
        --exclude-dir=fixtures \
        --exclude-dir=assets \
        | grep -v 'src/styles/tokens.css:' \
        | grep -vE '#(ph|root)[-a-z0-9]*' || true)

if [ -n "$hits" ]; then
  echo "hardcoded colors outside src/styles/tokens.css and src/fixtures/:"
  echo "$hits"
  exit 1
fi
echo "check-no-raw-hex: every color in ${target#"$here/"} comes from a token"
