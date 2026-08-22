#!/usr/bin/env bash
set -euo pipefail
# Stream handlers must cross IPC through Channel; events are notifications only.
if rg -n 'emit\(' apps/desktop/src-tauri/src apps/desktop/src/panes | rg -v 'RunFinished|TaskMoved|GuardrailTripped'; then
  echo 'high-frequency pane code must use Channel, not emit' >&2
  exit 1
fi
rg -q 'Channel<Vec<u8>>' apps/desktop/src-tauri/src/lib.rs
rg -q 'Channel<Event>' apps/desktop/src-tauri/src/lib.rs
