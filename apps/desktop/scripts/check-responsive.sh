#!/usr/bin/env bash
# The layout fills its host. Nothing is pinned to the mockup's 1440x900.
#
# The handoff says "fixed 1440x900, no responsive behaviour" — true of a picture
# of an app, and wrong for one. A pane pinned in px is a pane that overflows on a
# smaller window and strands whitespace on a larger one.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

# The window fills what it is given.
grep -q 'width: 100%' src/shell/shell.css && grep -q 'height: 100%' src/shell/shell.css || {
  echo "the window does not fill its host"
  fail=1
}
if grep -qE '(width|height): *(1440|900)px' src/shell/shell.css src/screens/screens.css; then
  echo "the mockup's 1440x900 is still pinned somewhere:"
  grep -nE '(width|height): *(1440|900)px' src/shell/shell.css src/screens/screens.css
  fail=1
fi

# Every pane width is a preference, not a constant. The rail and the small fixed
# chrome (dots, avatars, gutters, tracks) are exempt: those are glyph sizes.
panes='\.inbox-list|\.plan-list|\.plan-outputs|\.wiki-tree|\.wiki-side|\.wf-palette|\.wf-canvas|\.wf-inspector'
for sel in inbox-list plan-list plan-outputs wiki-tree wiki-side wf-palette wf-inspector; do
  block=$(awk -v s=".$sel {" 'index($0,s){f=1} f{print} f&&/^}/{exit}' src/screens/screens.css)
  echo "$block" | grep -q 'width: clamp(' || {
    echo ".$sel is a fixed width, not a preference"
    fail=1
  }
done

# A pane the reader resizes starts from a clamp too, and only becomes px on drag.
grep -q 'clamp(' src/panes/Resizable.tsx || {
  echo "Resizable pins its preferred width instead of clamping it"
  fail=1
}

# Card grids reflow rather than squeezing to nothing.
for grid in status-metrics tm-metrics type-grid hn-grid status-middle; do
  grep -q "\.$grid {.*auto-fit" src/screens/screens.css || {
    echo ".$grid does not reflow"
    fail=1
  }
done

# Scrollbars are thin and only appear on hover: a permanent chunky bar is what a
# layout that did not fit looks like.
grep -q 'scrollbar-width: thin' src/styles/app.css || {
  echo "scrollbars are not thinned"
  fail=1
}

if [ "$fail" -eq 0 ]; then
  echo "check-responsive: the window fills its host and every pane width is a preference"
fi
exit "$fail"
