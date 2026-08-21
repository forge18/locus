# locus-browse — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Browser service container per project on the project network | — | `cargo test -p locus-core browse::container_per_project` |
| 2 | Playwright driver over the project network | 1 | `cargo test -p locus-core browse::driver` |
| 3 | One browser context per run | 2 | `cargo test -p locus-core browse::context_per_run` |
| 4 | Assert two runs cannot see each other's cookies, storage or pages | 3 | `cargo test -p locus-core browse::contexts_are_isolated` |
| 5 | Project run script started at container start, backgrounded | — | `cargo test -p locus-core browse::app_started_by_container` |
| 6 | Readiness probe; `open` blocks until it passes | 5 | `cargo test -p locus-cli browse::open_waits_for_ready` |
| 7 | `locus browse open URL` relative to the run's own app | 6 | `cargo test -p locus-cli browse::open` |
| 8 | `click`, `fill`, `press` | 3 | `cargo test -p locus-cli browse::interactions` |
| 9 | `assert` with `--text`, `--visible`, `--count` | 3 | `cargo test -p locus-cli browse::assert` |
| 10 | `assert` exits non-zero with structured JSON on failure | 9 | `cargo test -p locus-cli browse::assert_exit_code` |
| 11 | A `Verify` node gates on `browse assert` directly | 10 | `cargo test -p locus-core browse::verify_can_gate` |
| 12 | `screenshot` landing as an image artifact | 3 | `cargo test -p locus-cli browse::screenshot` |
| 13 | Assert the agent performs no upload step | 12 | `cargo test -p locus-core browse::no_upload_step` |
| 14 | Screenshot appears on the run and the board card | 12 | `cargo test -p locus-core browse::artifact_on_card` |
| 15 | `record start\|stop` producing a recording artifact | 3 | `cargo test -p locus-cli browse::record` |
| 16 | Duration cap on recordings | 15 | `cargo test -p locus-core browse::record_duration_cap` |
| 17 | `console` and `network` returning text | 3 | `cargo test -p locus-cli browse::console_network` |
| 18 | No egress by default | 1 | `cargo test -p locus-core browse::no_egress_default` |
| 19 | A granted origin is a named project setting and is audited | 18 | `cargo test -p locus-core browse::granted_origin_audited` |
| 20 | Auto-waiting is real; docs blob advises against `sleep` | 8 | `cargo test -p locus-core browse::auto_waiting` |
| 21 | The browser survives one run ending while another is using it | 3 | `cargo test -p locus-core browse::survives_run_exit` |
