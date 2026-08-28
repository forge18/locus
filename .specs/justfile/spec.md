# justfile

**Milestone** M1.5 · **Depends on** completed M1 CI (`ci`)

## Purpose

One typed entrypoint for Locus's build steps. The same commands live in four places — the AGENTS.md
commands table, `.github/workflows/ci.yml` shell lines, `apps/desktop/package.json` scripts, and
`scripts/*.sh` — and nothing keeps them from drifting. A root `justfile` makes the recipes the single
source each surface calls, without rewriting the 1,700 existing `verify:` commands in `.specs/`.

## Governed by

- AGENTS.md — the commands table is the agent-facing contract surface
- [ci](.specs/ci/spec.md) — the six checks the `ci` recipe must reproduce
- AGENTS.md convention — every task ships a runnable `verify:`; recipes stay thin

## Contract

**Thin recipes, verbatim commands.** Each recipe wraps the exact command it replaces — `just lint` is
`cargo clippy --all-targets -- -D warnings`, nothing more. The raw command stays readable in the
recipe line; the justfile adds naming and discoverability, never behavior.

**The recipe set.** `setup`, `build`, `test`, `test-node`, `lint`, `typecheck`, `dev`, and `ci` (the
full CI sequence). `test-named` delegates to `scripts/run-named-test.sh` and preserves its
fail-if-filter-is-stale semantics. Recipes pass arguments through positionally and quoted, so
`::`-suffixed test paths survive.

**Three surfaces call the recipes.**

1. The AGENTS.md commands table lists `just` recipes; raw commands below it stay valid. This changes
   the materialized prompt prefix for every harness once — an accepted, one-time cache miss inherent
   to any AGENTS.md edit.
2. CI keeps one annotated step per check and installs `just` version-pinned
   (`extractions/setup-just@v2`). The locusd socket smoke keeps its inline shell: it is
   CI-environment logic, not a repo command.
3. Existing `.specs/*/verify:` commands are **not rewritten**. Raw commands remain canonical for every
   existing row; new rows may use `just`. The dual surface is accepted — recipes and raw commands are
   the same commands spelled twice, never two behaviors.

**[ci](.specs/ci/spec.md) amended** to name the justfile as how its six checks run.

**No behavior change.** Recipes change how commands are named, not what they do: no new flags, no new
defaults, no reordering. Byte-determinism is untouched — the justfile is static text with no
timestamps, run ids, or environment reads.

## Acceptance

1. `just --list` shows the full recipe set with one-line descriptions.
2. Every recipe runs the command it wraps with identical flags and exit codes.
3. `just test-named <pkg> <path>` fails non-zero when the filter matches nothing — never a silent
   green.
4. CI runs one step per check through recipes and stays green, with `just` version-pinned.
5. Every recipe in `just --list` appears in the AGENTS.md commands table.
6. No `.specs/*/tasks.md` verify row is rewritten by this feature.

## Open

- Whether agent container images include `just` in the baked allowlist. Until decided, containers keep
  running raw commands and the justfile is host/CI-only.
