#!/usr/bin/env bash
set -euo pipefail

metrics_file="crates/locus-core/src/services/metrics.rs"
if rg -n -i '\b(insert|update|delete|create[[:space:]]+table|alter[[:space:]]+table)\b' "$metrics_file"; then
  echo "metric projections must not add a write path" >&2
  exit 1
fi
printf 'metric projections are read-only: %s\n' "$metrics_file"
