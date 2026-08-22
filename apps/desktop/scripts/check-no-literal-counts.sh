#!/usr/bin/env bash
# Every count on the Workshop screens comes from harnesses/*.toml through the
# generated fixture. A literal here is wrong the next time a harness is registered
# — the handoff's own copy said 88/27 while the files said 88/29, which is exactly
# the failure this prevents.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

# Numeric literals in the screen source, ignoring geometry (px, widths, sizes),
# array indices, and the comments that explain the rule.
hits=$(grep -nE '[^A-Za-z0-9_.$-]([0-9]{2,})[^A-Za-z0-9_%px]' \
        src/screens/workshop/ExtensionsView.tsx \
        src/screens/workshop/HarnessesView.tsx \
        | grep -v '^\s*//' \
        | grep -vE '^\S+: *(//|\*|/\*)' \
        | grep -vE "'[0-9]+px'|width|height|size=|style=" || true)

if [ -n "$hits" ]; then
  echo "numeric literals in a Workshop screen — every count must be computed:"
  echo "$hits"
  fail=1
fi

# And the screens must actually read the generated registry.
for f in src/screens/workshop/ExtensionsView.tsx src/screens/workshop/HarnessesView.tsx; do
  grep -q "from '../../data/harnesses'" "$f" || {
    echo "$f does not read the computed registry"
    fail=1
  }
done

# The counts the spec names are in the generated file, and nowhere else.
grep -q 'HARNESS_COUNT = 11' src/fixtures/generated/harnesses.ts || {
  echo "the generated registry does not report 11 harnesses"
  fail=1
}
grep -q 'DOWNGRADE_COUNT = 29' src/fixtures/generated/harnesses.ts || {
  echo "the generated registry does not report 29 downgrades"
  fail=1
}

if [ "$fail" -eq 0 ]; then
  echo "check-no-literal-counts: every Workshop count is computed from the registry"
fi
exit "$fail"
