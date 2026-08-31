#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
if [[ "$(uname -s)" == "Linux" && -z "${DISPLAY:-}" ]]; then
  if ! command -v xvfb-run >/dev/null 2>&1; then
    printf '%s\n' 'DESKTOP_INTEGRATION_UNSUPPORTED: Linux requires xvfb-run when DISPLAY is unset.' >&2
    exit 0
  fi
  exec xvfb-run -a --server-args='-screen 0 1440x900x24' \
    node "$script_dir/test-desktop-integration.mjs"
fi

exec node "$script_dir/test-desktop-integration.mjs"
