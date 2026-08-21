#!/usr/bin/env bash
set -euo pipefail

if rg -n --glob '*.rs' --glob '*.sh' 'locus[[:space:]]+lint' crates/locus-cli/src/hook.rs harnesses; then
  echo 'locus lint must not run from a hook' >&2
  exit 1
fi
