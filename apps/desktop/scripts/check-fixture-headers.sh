#!/usr/bin/env bash
# Every fixture module opens with the schema it draws from and the Tauri command
# that will replace it. The header is the reconciliation note: at M1 the swap is a
# wiring change, and this is what says where the wire goes.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
for f in $(find src/fixtures -name '*.ts' | sort); do
  case "$(basename "$f")" in
    index.ts) continue ;;
  esac

  head -1 "$f" | grep -qE '^// schema: ' || {
    echo "$f: line 1 must be '// schema: <schema>.<table> [+ …]'"
    fail=1
  }
  head -2 "$f" | tail -1 | grep -qE '^// replaced by: ' || {
    echo "$f: line 2 must be '// replaced by: invoke(\"…\") [+ …]'"
    fail=1
  }
done

if [ "$fail" -eq 0 ]; then
  echo "check-fixture-headers: every fixture names its schema and its future command"
fi
exit "$fail"
