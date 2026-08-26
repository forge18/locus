#!/usr/bin/env bash
set -euo pipefail

if rg -n -i '\b(poll|polling|poller|sync[[:space:]]+job)\b' crates/locus-core/src/forge.rs; then
  echo "forge adapters must be webhook-driven, not polling-driven" >&2
  exit 1
fi
printf 'forge integration has no polling loop\n'
