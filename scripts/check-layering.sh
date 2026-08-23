#!/usr/bin/env bash
# Keep the locus-core layers pointing one way.
#
# The module tree mirrors PLAN.md §Process topology, and that only stays true if something
# checks it. Convention alone lasted until M1 closed and then stopped: `Store` ended up
# assembled from seven feature modules, and `store` and `sandbox` imported each other.
#
# Tests are exempt. A test may reach across layers to build a fixture; production may not.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
core_src="$repo_root/crates/locus-core/src"

if [[ ! -d "$core_src" ]]; then
    printf 'expected locus-core source under %s\n' "$repo_root" >&2
    exit 1
fi

# Blank every `#[cfg(test)]` / `#[test]` item, keeping line numbers, so a reported
# violation points at the real line. Same strip the harness-name check uses.
strip_test_items='
/^#\[(test|tokio::test)\]/ || /^#\[cfg\((test|all\(test)/ { in_test = 1; opened = 0; print ""; next }
in_test == 1 && opened == 0 {
    print ""
    if (/\{[[:space:]]*$/) { opened = 1; next }
    if (/^#\[/)            { next }
    if (/;[[:space:]]*$/)  { in_test = 0 }
    next
}
in_test == 1 {
    print ""
    if (/^\}/) { in_test = 0 }
    next
}
{ print }
'

violations=0

# rule <description> <path glob to search> <path glob to skip> <forbidden pattern>
rule() {
    description=$1
    include=$2
    exclude=$3
    pattern=$4

    while IFS= read -r source; do
        # shellcheck disable=SC2053  # glob matching is the point: exclude is a path glob
        if [[ -n "$exclude" && "$source" == $exclude ]]; then
            continue
        fi
        matches=$(awk "$strip_test_items" "$source" | grep -nE "$pattern" || true)
        if [[ -n "$matches" ]]; then
            printf '%s\n' "$matches" | sed "s|^|${source#"$repo_root"/}:|" >&2
            printf '  ^ %s\n' "$description" >&2
            violations=1
        fi
    done < <(find "$include" -type f -name '*.rs' -print 2>/dev/null | LC_ALL=C sort)
}

# Deriving `sqlx::Type` maps a newtype onto a column; it is not a query. The rule is
# about running SQL, so match the query APIs rather than the crate name.
rule 'only store/ may run a query — every other layer goes through a Store method' \
    "$core_src" "$core_src/store/*" 'sqlx::query|query_as|query_scalar|\.fetch_(one|all|optional)\(|\.execute\('

rule 'services/ is shared services; it must not reach into run supervision or the sandbox' \
    "$core_src/services" "" 'crate::(runtime|sandbox)'

rule 'sandbox/ must not name the store — the credential proxy records through a sink' \
    "$core_src/sandbox" "" 'crate::store'

rule 'testkit is test support; production code must not import it' \
    "$core_src" "$core_src/testkit/*" 'crate::testkit'

if [[ "$violations" -ne 0 ]]; then
    printf 'locus-core layering violated; see PLAN.md §Process topology\n' >&2
    exit 1
fi

printf 'locus-core layers point one way\n'
