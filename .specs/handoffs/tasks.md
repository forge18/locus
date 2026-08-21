# handoffs — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Handoff payload type with all eight fields | — | `cargo test -p locus-core handoff::payload_shape` |
| 2 | Persist as an artifact linked to both sessions | 1 | `cargo test -p locus-core handoff::persists_as_artifact` |
| 3 | `handed_off_from` on the successor session | 2 | `cargo test -p locus-core handoff::links_sessions` |
| 4 | Traverse the chain in both directions | 3 | `cargo test -p locus-core handoff::chain_traversable` |
| 5 | `locus handoff <agent> --why` ending the current session | 1 | `cargo test -p locus-cli handoff::ends_session` |
| 6 | Open the successor on the same task and branch | 5 | `cargo test -p locus-core handoff::same_task_and_branch` |
| 7 | Prime the successor from the payload | 6 | `cargo test -p locus-core handoff::primes_from_payload` |
| 8 | Assert the predecessor's transcript is not injected | 7 | `cargo test -p locus-core handoff::no_transcript_replay` |
| 9 | Require non-empty `attempted[]` when the trigger was stuck | 1 | `cargo test -p locus-core handoff::attempted_required_when_stuck` |
| 10 | Trigger: guardrail kill-and-reassign produces a handoff | 5 | `cargo test -p locus-core handoff::from_guardrail` |
| 11 | Trigger: context exhausted | 5 | `cargo test -p locus-core handoff::from_context_exhaustion` |
| 12 | Trigger: workflow graph role change | 5 | `cargo test -p locus-core handoff::from_graph` |
| 13 | Trigger: human reassignment | 5 | `cargo test -p locus-core handoff::from_human` |
| 14 | All four triggers produce the same artifact shape | 10,11,12,13 | `cargo test -p locus-core handoff::one_shape_four_triggers` |
| 15 | Assert no return path to the predecessor exists | 6 | `cargo test -p locus-core handoff::does_not_return` |
| 16 | Reference predecessor artifacts rather than copying them | 2 | `cargo test -p locus-core handoff::references_not_copies` |
| 17 | Surface the handoff summary in the Agents screen's stuck footer | 10 | `pnpm -C apps/desktop test -- agents/handoff-summary` |
