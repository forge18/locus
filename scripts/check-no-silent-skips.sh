#!/usr/bin/env bash
set -euo pipefail

# Docker-backed tests must opt in explicitly, and every ignored test must explain why.
if rg -n '#\[ignore\][[:space:]]*$' crates >/dev/null; then
  echo 'ignored tests must explain why; Docker-backed tests must declare their requirement' >&2
  exit 1
fi

for required in \
  'crates/locus-core/src/sandbox/docker.rs:requires Docker daemon' \
  'crates/locus-core/src/harness/materialize/mod.rs:requires Docker image' \
  'crates/locus-core/tests/smoke.rs:requires Docker for the future live-harness smoke'; do
  path=${required%%:*}
  message=${required#*:}
  if ! grep -Fq "$message" "$path"; then
    echo "missing explicit Docker marker in $path" >&2
    exit 1
  fi
done
