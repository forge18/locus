# linters — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Linter directory format: `<name>.sh` plus `<name>.md` | — | `cargo test -p locus-core lint::format` |
| 2 | Refuse a `.sh` with no `.md`, naming the missing rule file | 1 | `cargo test -p locus-core lint::rule_file_required` |
| 3 | Materialize into `/locus/config/linters/` via the `dir` strategy | 1 | `cargo test -p locus-core lint::materializes` |
| 4 | Assert all twelve harnesses produce an identical linters tree | 3 | `cargo test -p locus-core lint::identical_across_harnesses` |
| 5 | Discovery and execution of every linter | 3 | `cargo test -p locus-cli lint::runs_all` |
| 6 | `--only NAME` running exactly one | 5 | `cargo test -p locus-cli lint::only` |
| 7 | `--changed` scoping to the run's diff | 5 | `cargo test -p locus-cli lint::changed` |
| 8 | Non-zero exit on failure | 5 | `cargo test -p locus-cli lint::exit_code` |
| 9 | Print the rule `.md` alongside the check's message on failure | 8 | `cargo test -p locus-cli lint::prints_the_rule` |
| 10 | A `Verify` node gates on the exit code directly | 8 | `cargo test -p locus-core lint::verify_can_gate` |
| 11 | Capture stdout as evidence for a board transition | 8 | `cargo test -p locus-core lint::output_is_evidence` |
| 12 | Assert `locus lint` is never invoked from a hook | 5 | `bash scripts/check-lint-not-in-hooks.sh` |
| 13 | Decide and implement path-glob scoping, or record that it is directory-only | 1 | `cargo test -p locus-core lint::scoping` |
| 14 | Wire the Extensions screen's linters count to real linters | 3 | `pnpm -C apps/desktop test -- extensions/linters-count` |
