#!/usr/bin/env bash
# Every exported type names the Postgres schema it mirrors, in the doc comment
# directly above it. A type with no schema is a type that got its shape from a
# screen, which is the failure mode this whole feature exists to avoid.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
for f in src/types/*.ts; do
  [ "$(basename "$f")" = "index.ts" ] && continue

  head -3 "$f" | grep -q 'Mirrors the `[a-z]*` Postgres schema' || {
    echo "$f: no header naming the Postgres schema it mirrors"
    fail=1
  }

  # Only the comment block immediately above the declaration counts. Anything
  # else and a type could borrow its neighbour's citation.
  awk -v file="$f" '
    /^[[:space:]]*(\/\*|\*|\/\/)/ { block = block $0 "\n"; next }
    /^[[:space:]]*$/              { next }
    /^export (interface|type|const) / {
      if (block !~ /@schema [a-z]+/) {
        printf "%s:%d: exported type does not name its schema\n%s\n", file, NR, $0
        bad = 1
      }
      block = ""
      next
    }
    { block = "" }
    END { exit bad }
  ' "$f" || fail=1
done

if [ "$fail" -eq 0 ]; then echo "check-types-cite-schema: every exported type names its schema"; fi
exit "$fail"
