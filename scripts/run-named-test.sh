#!/usr/bin/env bash
# Run one named test and fail if the filter matched nothing.
#
# `cargo test <filter>` exits 0 when no test matches, so a renamed, moved, or misspelled
# test silently stops running and CI stays green. Every CI step that names a single test
# goes through here.
set -euo pipefail

if [[ $# -lt 2 ]]; then
    printf 'usage: run-named-test.sh <package> <test-path> [harness args...]\n' >&2
    exit 1
fi

package=$1
test_path=$2
shift 2

if ! output=$(cargo test -p "$package" "$test_path" -- --exact "$@" 2>&1); then
    printf '%s\n' "$output" >&2
    exit 1
fi
printf '%s\n' "$output"

if ! printf '%s\n' "$output" | grep -qE '^test result: ok\. [1-9]'; then
    printf 'no test matched %s in %s; the filter is stale\n' "$test_path" "$package" >&2
    exit 1
fi
