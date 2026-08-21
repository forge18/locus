#!/usr/bin/env bash
# Exactly two @keyframes exist in the whole app: pulse and blink.
set -euo pipefail
cd "$(dirname "$0")/.."

names=$(grep -rhoE '@keyframes\s+[A-Za-z0-9_-]+' src --include='*.css' --include='*.ts' --include='*.tsx' \
        | awk '{print $2}' | sort)
count=$(echo "$names" | sed '/^$/d' | wc -l | tr -d ' ')

if [ "$count" != "2" ] || [ "$(echo "$names" | tr '\n' ' ')" != "blink pulse " ]; then
  echo "expected exactly two keyframes (blink, pulse); found $count: $(echo "$names" | tr '\n' ' ')"
  exit 1
fi
echo "check-keyframes: exactly two — pulse and blink"
