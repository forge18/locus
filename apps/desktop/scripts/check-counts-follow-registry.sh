#!/usr/bin/env bash
# Registering a thirteenth harness changes both Workshop screens with no source
# edit. This adds one, regenerates, checks the figures moved, and puts it back.
set -euo pipefail
cd "$(dirname "$0")/.."

GENERATED=src/fixtures/generated/harnesses.ts
PROBE=../../harnesses/zz-probe.toml
BACKUP=$(mktemp)
cp "$GENERATED" "$BACKUP"

# shellcheck disable=SC2329 # invoked by EXIT trap
restore() {
  rm -f "$PROBE"
  cp "$BACKUP" "$GENERATED"
  rm -f "$BACKUP"
}
trap restore EXIT

before_harnesses=$(grep -oE 'HARNESS_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')
before_entries=$(grep -oE 'ENTRY_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')
before_downgrades=$(grep -oE 'DOWNGRADE_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')

# A thirteenth harness: all eight types declared, two of them downgraded.
cat > "$PROBE" <<'TOML'
name   = "zz-probe"
binary = "zz-probe"
detect = ["--version"]

[launch]
argv = []
tui  = false

[telemetry]
source = "session-log"
emits  = ["session_start", "user", "assistant", "session_end"]

[models]
flag      = "--model"
list_argv = []

[layout]
agents        = { via = "dir", dir = "/locus/config/agents", format = "markdown+frontmatter" }
commands      = { via = "dir", dir = "/locus/config/commands", format = "markdown+frontmatter" }
hooks         = { via = "core-driven", weaker_than_native = "no hook mechanism; Locus fires them at the container boundary only" }
linters       = { via = "dir", dir = "/locus/config/linters", format = "shell+markdown" }
output-styles = { via = "merged-into", target = "context", weaker_than_native = "appended to context; a native style REPLACES the prompt's communication section" }
rules         = { via = "dir", dir = "/locus/config/rules", format = "markdown+frontmatter" }
skills        = { via = "dir", dir = "/locus/config/skills", format = "markdown+frontmatter" }
context       = { via = "file", file = "/locus/config/AGENTS.md" }
TOML

pnpm exec tsx scripts/gen-harness-fixtures.ts >/dev/null

after_harnesses=$(grep -oE 'HARNESS_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')
after_entries=$(grep -oE 'ENTRY_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')
after_downgrades=$(grep -oE 'DOWNGRADE_COUNT = [0-9]+' "$GENERATED" | grep -oE '[0-9]+')

fail=0
[ "$after_harnesses" = "$((before_harnesses + 1))" ] || {
  echo "harness count did not follow the registry: $before_harnesses -> $after_harnesses"
  fail=1
}
[ "$after_entries" = "$((before_entries + 8))" ] || {
  echo "entry count did not follow the registry: $before_entries -> $after_entries"
  fail=1
}
[ "$after_downgrades" = "$((before_downgrades + 2))" ] || {
  echo "downgrade count did not follow the registry: $before_downgrades -> $after_downgrades"
  fail=1
}

# And nothing in the screen source had to change for that to happen.
if ! git diff --quiet -- src/screens/workshop/ 2>/dev/null; then
  echo "the Workshop screens changed; the counts should have come from the registry"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "check-counts-follow-registry: $before_harnesses -> $after_harnesses harnesses, $before_downgrades -> $after_downgrades downgrades, no source edit"
fi
exit "$fail"
