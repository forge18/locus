#!/usr/bin/env bash
# Verify that HEAD is the test-only commit in a RED/GREEN TDD pair.
set -euo pipefail

parent=$(git rev-parse HEAD^)
subject=$(git log -1 --format=%s)

if [[ ! $subject =~ ^test(\(.+\))?: ]]; then
  echo "HEAD must be a Conventional Commit test commit; got: $subject" >&2
  exit 1
fi

while IFS= read -r path; do
  case "$path" in
    crates/*/src/*|apps/desktop/src/*|migrations/*|scripts/*)
      case "$path" in
        */test/*|*/tests/*|*_test.rs|*.test.ts|*.test.tsx) ;;
        *) echo "RED commit changes non-test file: $path" >&2; exit 1 ;;
      esac
      ;;
  esac
done < <(git diff --name-only "$parent" HEAD)

echo "RED commit shape verified: $subject"
