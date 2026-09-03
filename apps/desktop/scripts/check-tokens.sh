#!/usr/bin/env bash
# Every theme value and invariant geometry token is declared in the token source.
set -euo pipefail
cd "$(dirname "$0")/.."

FILE=src/styles/tokens.css
fail=0

theme_body() {
  local theme="$1"
  awk -v selector="[data-theme=\"$theme\"]" '
    index($0, selector " {") == 1 { inside = 1; next }
    inside && /^}/ { exit }
    inside { print }
  ' "$FILE"
}

check_theme() {
  local theme="$1"
  shift
  local body token value
  body=$(theme_body "$theme")
  if [ -z "$body" ]; then
    echo "MISSING theme: $theme"
    fail=1
    return
  fi
  while [ "$#" -gt 0 ]; do
    token="$1"
    value="$2"
    shift 2
    if ! grep -qF -- "$token: $value;" <<<"$body"; then
      echo "MISSING or WRONG in $theme: $token should be '$value'"
      fail=1
    fi
  done
}

check_theme dark \
  --surface-ground '#1d2731' \
  --surface-chrome '#151d25' \
  --surface-raised '#22303c' \
  --surface-selected '#293947' \
  --surface-elevated '#314454' \
  --text-primary '#eef2f6' \
  --action-attention '#ffbb39' \
  --status-working '#9184d9' \
  --status-success '#68ad91' \
  --status-danger '#df8a7d'

check_theme light \
  --surface-ground '#f3f6f8' \
  --surface-chrome '#e8eef3' \
  --surface-raised '#ffffff' \
  --surface-selected '#e3edf5' \
  --surface-elevated '#e6eef6' \
  --text-primary '#16212b' \
  --action-attention '#9a5b00' \
  --status-working '#675bb0' \
  --status-success '#237250' \
  --status-danger '#a7372d'

for token in \
  '--fm: "JetBrains Mono", ui-monospace, Menlo, monospace;' \
  '--g-6: 16px;' \
  '--g-7: 18px;' \
  '--g-8: 20px;' \
  '--g-9: 24px;' \
  '--g-10: 32px;'; do
  grep -qF -- "$token" "$FILE" || {
    echo "MISSING root token: $token"
    fail=1
  }
done

if [ "$fail" -eq 0 ]; then
  echo "check-tokens: dark/light semantic values and invariant geometry are declared"
fi
exit "$fail"
