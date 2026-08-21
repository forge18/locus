#!/usr/bin/env bash
# src/ui/ is chrome. Split panes, the file tree, and virtual lists are product —
# they live in panes/ and their screens, where they can know what they are showing.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

# No file named for a product surface.
# `EmptyPane` is a state, not the pane manager — match the surfaces, not the word.
banned_files=$(ls src/ui | grep -iE 'split|resiz|panemanager|pane-manager|filetree|file-tree|tree|virtual|canvas|terminal|editor|diff' || true)
if [ -n "$banned_files" ]; then
  echo "product surfaces do not belong in src/ui/:"
  echo "$banned_files"
  fail=1
fi

# No import that drags one in.
banned_imports=$(grep -rnE "from '[^']*(solid-flow|xterm|codemirror|@tanstack/solid-virtual|solid-virtual|split)" src/ui || true)
if [ -n "$banned_imports" ]; then
  echo "src/ui/ imports a product dependency:"
  echo "$banned_imports"
  fail=1
fi

if [ "$fail" -eq 0 ]; then echo "check-ui-scope: src/ui/ is chrome only"; fi
exit "$fail"
