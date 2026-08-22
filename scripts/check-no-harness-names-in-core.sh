#!/usr/bin/env bash
# Keep locus-core harness-neutral: registry declarations, not Rust production code,
# identify supported harnesses.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
core_src="$repo_root/crates/locus-core/src"
harness_dir="$repo_root/harnesses"

if [[ ! -d "$core_src" || ! -d "$harness_dir" ]]; then
    printf 'expected locus-core source and harness registry under %s\n' "$repo_root" >&2
    exit 1
fi

harness_names=$(find "$harness_dir" -type f -name '*.toml' -exec basename {} .toml \; | LC_ALL=C sort | paste -sd '|' -)
if [[ -z "$harness_names" ]]; then
    printf 'no harness declarations found in %s\n' "$harness_dir" >&2
    exit 1
fi

pattern="(^|[^[:alnum:]_])(${harness_names})([^[:alnum:]_]|$)"
violations=0

while IFS= read -r source; do
    case "$source" in
    "$core_src/registry.rs")
        # Registry unit tests may name fixture declarations. Production code ends here.
        content=$(awk '/^#\[cfg\(test\)\]/{ exit } { print }' "$source")
        ;;
    "$core_src/registry/fixtures/"*)
        # Fixtures may name a harness so registry tests can exercise real declarations.
        continue
        ;;
    *)
        content=$(cat "$source")
        ;;
    esac

    matches=$(printf '%s\n' "$content" | grep -nE "$pattern" || true)
    if [[ -n "$matches" ]]; then
        printf '%s\n' "$matches" | sed "s|^|$source:|" >&2
        violations=1
    fi
done < <(find "$core_src" -type f -name '*.rs' -print | LC_ALL=C sort)

if [[ "$violations" -ne 0 ]]; then
    printf 'locus-core production code must not name a harness; use the registry instead\n' >&2
    exit 1
fi

printf 'locus-core contains no harness names outside registry tests and fixtures\n'
