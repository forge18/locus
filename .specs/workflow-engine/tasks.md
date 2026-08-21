# workflow-engine — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `workflow_defs` table: `graph` JSONB, `spec` JSONB, version | — | `cargo test -p locus-core workflow::defs_table` |
| 2 | Compile `graph` → `spec`, both written in one transaction | 1 | `cargo test -p locus-core workflow::compile_together` |
| 3 | Assert a spec/graph disagreement is impossible by construction | 2 | `cargo test -p locus-core workflow::cannot_disagree` |
| 4 | Supervisor walks the `spec` | 2 | `cargo test -p locus-core workflow::walks_spec` |
| 5 | `Loop` reset starting a fresh run in the same session | 4 | `cargo test -p locus-core workflow::reset_same_session` |
| 6 | Memory, branch and task carry across a reset | 5 | `cargo test -p locus-core workflow::state_survives_reset` |
| 7 | `Verify` in a fresh container from the agent's image on the run's branch | 4 | `cargo test -p locus-core workflow::verify_fresh_container` |
| 8 | Assert verify fails when the change exists only in the agent's container | 7 | `cargo test -p locus-core workflow::verify_is_not_local` |
| 9 | Capture exit code, stdout and stderr as evidence | 7 | `cargo test -p locus-core workflow::verify_evidence` |
| 10 | `Condition` expression parser over the ten operands | — | `cargo test -p locus-core condition::parses` |
| 11 | Operators and parentheses | 10 | `cargo test -p locus-core condition::operators` |
| 12 | Evaluate as a `WHERE` clause against the run | 10 | `cargo test -p locus-core condition::evaluates` |
| 13 | Refuse an unknown operand at compile time | 10 | `cargo test -p locus-core condition::rejects_unknown_operand` |
| 14 | Re-evaluation against stored events is reproducible | 12 | `cargo test -p locus-core condition::reproducible` |
| 15 | Assert no I/O, no model, no unbounded loop in evaluation | 12 | `cargo test -p locus-core condition::is_total` |
| 16 | `Gate` node: human and reviewer-agent variants | 4 | `cargo test -p locus-core workflow::gate` |
| 17 | `Goal` as the approval gate before the loop may run | 4 | `cargo test -p locus-core workflow::goal_gates_the_loop` |
| 18 | Arbiter agent classifying a failure into four classes | 7 | `cargo test -p locus-core arbiter::classifies` |
| 19 | Classification recorded as a column on the iteration | 18 | `cargo test -p locus-core arbiter::column_on_iteration` |
| 20 | Bug: retry and promote the failing check into the regression set | 18 | `cargo test -p locus-core arbiter::bug_promotes_check` |
| 21 | Noise: recalibrate and do not decrement the iteration budget | 18 | `cargo test -p locus-core arbiter::noise_is_free` |
| 22 | Spec gap: emit a new task for the delta and leave the workflow | 18 | `cargo test -p locus-core arbiter::spec_gap_exits` |
| 23 | Ambiguity: refine then restart, never retry the implementation | 18 | `cargo test -p locus-core arbiter::ambiguity_restarts` |
| 24 | Spec-gap and ambiguity rates as queries | 19 | `cargo test -p locus-core arbiter::rates_are_queries` |
| 25 | `locus ralph --goal --verify` running a loop with no canvas | 4 | `cargo test -p locus-cli ralph::runs` |
| 26 | Assert no model is invoked in the orchestration path | 12,4 | `cargo test -p locus-core workflow::no_model_in_orchestration` |
