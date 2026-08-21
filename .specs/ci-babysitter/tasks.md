# ci-babysitter — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Detect a failing pipeline on a run's branch | — | `cargo test -p locus-core babysit::detects_failure` |
| 2 | Fetch and compact the pipeline logs | 1 | `cargo test -p locus-core babysit::fetches_logs` |
| 3 | Hand the logs to an agent in a container | 2 | `cargo test -p locus-core babysit::dispatches_agent` |
| 4 | The agent pushes a fix to the branch | 3 | `cargo test -p locus-core babysit::pushes_fix` |
| 5 | Assert the babysitter never merges | 4 | `cargo test -p locus-core babysit::never_merges` |
| 6 | Bound retries using the existing guardrail config | 4 | `cargo test -p locus-core babysit::bounded_by_guardrails` |
| 7 | Assert no private retry counter exists | 6 | `cargo test -p locus-core babysit::no_second_counter` |
| 8 | Route CI failures through the arbiter | 2 | `cargo test -p locus-core babysit::classifies` |
| 9 | A noise-classified failure does not spend the budget | 8 | `cargo test -p locus-core babysit::noise_is_free` |
| 10 | Escalate on budget exhaustion as an inbox item | 6 | `cargo test -p locus-core babysit::escalates` |
| 11 | The escalation carries what was tried | 10 | `cargo test -p locus-core babysit::escalation_carries_attempts` |
| 12 | A deliberately broken build is fixed within budget | 4 | `cargo test -p locus-core babysit::fixes_real_break -- --ignored` |
| 13 | A deliberately unfixable build escalates rather than looping | 10 | `cargo test -p locus-core babysit::gives_up_cleanly -- --ignored` |
| 14 | Decide workflow versus supervisor behavior, and implement it | 1 | `cargo test -p locus-core babysit::shape_decided` |
