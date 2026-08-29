# justfile — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Root `justfile` with `setup`, `build`, `test`, `test-node`, `lint`, `typecheck`, and `dev`, each wrapping the current command verbatim | — | `just --list` names all seven; `just lint` exits clean with clippy's exact flags |
| 2 | `test-named` recipe delegating to `scripts/run-named-test.sh`, arguments quoted through | 1 | `just test-named locus-core harness::materialize::ci_determinism` passes; `just test-named locus-core no::such::test` exits non-zero |
| 3 | `ci` recipe reproducing the CI sequence verbatim, including `--ignored` canary steps | 1, 2 | `just -n ci` prints every step's command from `.github/workflows/ci.yml`; full `just ci` passes on a Docker-capable host |
| 4 | AGENTS.md commands table swapped to `just` recipes, raw commands noted as still valid | 1 | every recipe in `just --list` appears in the AGENTS.md table |
| 5 | `.specs/ci/spec.md` contract names the justfile as how the six checks run | 1 | `grep -l justfile .specs/ci/spec.md` |
| 6 | CI workflow: pinned `extractions/setup-just@v2`, each step's `run:` calls its recipe, locusd smoke stays inline | 1, 3 | `act -n -W .github/workflows/ci.yml`; the PR's CI run is green |
