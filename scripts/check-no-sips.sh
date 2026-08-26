#!/usr/bin/env bash
set -euo pipefail
if rg -n '\bsips\b' crates/locus-core/src/services/media.rs crates/locus-cli/src; then
  echo 'sips is not supported; use Rust image/ffmpeg derivation' >&2
  exit 1
fi
printf '%s\n' 'no sips dependency'
