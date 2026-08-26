#!/usr/bin/env bash
# One resolver, seven callers. A component that sets `view` itself is an eighth
# navigation path, and it will drift from the other seven — the palette, search,
# inbox items, board-card links, artifact comments, deep links, and a detached
# window's identity all have to agree about what a locator means.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

# Only the nav store may hold view state.
holders=$(grep -rn 'createSignal<View>\|createSignal<NavTarget>' src \
           --include='*.ts' --include='*.tsx' \
           | grep -v '^src/nav/store.ts:' || true)
if [ -n "$holders" ]; then
  echo "view state is held outside the nav store:"
  echo "$holders"
  fail=1
fi

# Internal screens may have local tabs; only the navigation target setter is guarded.
setters=$(grep -rn 'setTarget(' src --include='*.ts' --include='*.tsx' \
           | grep -v '^src/nav/' || true)
if [ -n "$setters" ]; then
  echo "something outside src/nav/ sets the view directly:"
  echo "$setters"
  fail=1
fi

# Navigation goes through the store's go/open, never through resolve() by hand.
callers=$(grep -rnE '(^|[^.[:alnum:]_])resolve\(' src --include='*.ts' --include='*.tsx' \
           | grep -v '^src/nav/' || true)
if [ -n "$callers" ]; then
  echo "something outside src/nav/ calls the resolver directly instead of nav.open():"
  echo "$callers"
  fail=1
fi

# The store itself must be the only place `view` is assigned.
if [ "$(grep -c 'setTarget(' src/nav/store.ts)" -gt 3 ]; then
  echo "the store assigns the target in more than the three documented places"
  fail=1
fi

if [ "$fail" -eq 0 ]; then echo "check-single-resolver: one resolver, and the store is its only writer"; fi
exit "$fail"
