# plan-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Replace the nine-stage core enum with the seven canonical stages | — | `cargo test -p locus-core planning::seven_stages` |
| 2 | Make Recommend advance directly to Decompose | 1 | `cargo test -p locus-core planning::recommend_advances_to_decompose` |
| 3 | Preserve `EditableSpec`, requirements, approved plans, card modes, decompositions, and board cards through the stage migration | 1 | `cargo test -p locus-core planning::stage_migration_preserves_plan_artifacts` |
| 4 | Register all seven stage locators and reject Audit and Override stage routes | 1 | `pnpm -C apps/desktop test -- plan/seven-stage-locators` |
| 5 | Build the clickable seven-stage strip and Back/Next stepper | 4 | `pnpm -C apps/desktop test -- plan/stage-strip-and-stepper` |
| 6 | Disable Back on Inputs and Next on Approved | 5 | `pnpm -C apps/desktop test -- plan/stepper-boundaries` |
| 7 | Build the All plans rail with In progress, rejected Drafts, and Approved sections | 4 | `pnpm -C apps/desktop test -- plan/all-plans-rail` |
| 8 | Open a new plan at Inputs with no prefilled goal | 7 | `pnpm -C apps/desktop test -- plan/new-plan-inputs` |
| 9 | Build the persistent Outputs rail and jump controls | 4 | `pnpm -C apps/desktop test -- plan/outputs-rail` |
| 10 | Drive the Outputs recommendation card and Recommend chip from one spec state | 9 | `pnpm -C apps/desktop test -- plan/recommendation-single-source` |
| 11 | Build Inputs with required goal, project, and attached repositories | 4 | `pnpm -C apps/desktop test -- plan/inputs` |
| 12 | Index symbols, call graph, and history before Converse becomes reachable | 11 | `cargo test -p locus-core planning::orient_before_converse` |
| 13 | Render Orient progress and prevent its direct-stage jump before indexing finishes | 5,12 | `pnpm -C apps/desktop test -- plan/orient` |
| 14 | Embed the agent panel in Converse with workflow suppressed | 13 | `pnpm -C apps/desktop test -- plan/converse-agent-panel` |
| 15 | Render researcher, interviewer, auditor, and scope-decision messages inline | 14 | `pnpm -C apps/desktop test -- plan/converse-inline-roles` |
| 16 | Run Synthesis as requirements draft then unsupported-clause removal | 12 | `cargo test -p locus-core planning::two_pass_synthesis` |
| 17 | Carry unresolved synthesis gaps forward as `open[n]` | 16 | `cargo test -p locus-core planning::synthesis_carries_open_gaps` |
| 18 | Render Synthesis pass state and its carried gap count | 16,17 | `pnpm -C apps/desktop test -- plan/synthesis` |
| 19 | Render stable requirement ids, RFC-2119 blocks, and five-section outline | 17 | `pnpm -C apps/desktop test -- plan/recommend-requirements` |
| 20 | Add, mark resolved, and mark board-carried requirements without changing ids | 19 | `pnpm -C apps/desktop test -- plan/recommend-requirement-actions` |
| 21 | Save only changed requirements and re-synthesise them | 20 | `cargo test -p locus-core planning::resynthesises_changed_requirements` |
| 22 | Render Recommend version, unsaved state, confidence, history, revert, and save controls | 19,21 | `pnpm -C apps/desktop test -- plan/recommend-editor` |
| 23 | Compute card count only through `CardMode::card_count` for all three decomposition modes | — | `cargo test -p locus-core planning::card_mode_count` |
| 24 | Store plan-level Workflow/Harness/Model/Effort routing defaults | 23 | `cargo test -p locus-core planning::decomposition_routing_defaults` |
| 25 | Store per-task routing overrides and Reset to defaults | 24 | `cargo test -p locus-core planning::decomposition_task_overrides` |
| 26 | Disable Model and Effort until a Harness is chosen and show `auto-route` | 24 | `pnpm -C apps/desktop test -- plan/decompose-routing-defaults` |
| 27 | Render three card modes, task table, dependency column, and card-count footer | 23,25 | `pnpm -C apps/desktop test -- plan/decompose` |
| 28 | Render per-task overrides and reset without changing plan defaults | 25,27 | `pnpm -C apps/desktop test -- plan/decompose-task-overrides` |
| 29 | Render the Approved summary, four stat cards, stage log, and created-cards table | 27 | `pnpm -C apps/desktop test -- plan/approved` |
| 30 | Restrict Approved actions to Start a new plan and Open the board | 29 | `pnpm -C apps/desktop test -- plan/approved-actions` |
| 31 | Route post-approval spec edits through the board, not the Plan screen | 29 | `pnpm -C apps/desktop test -- plan/post-approval-edits` |
| 32 | Add the stage-list supersession pointer to `planning-module/spec.md` | 1 | `grep -q "Superseded by.*plan-revision" .specs/planning-module/spec.md` |
| 33 | Remove Audit and Override as stage labels, tabs, and routes from Plan sources | 1,4 | `! grep -rqi "Audit" apps/desktop/src/screens/plan crates/locus-core/src/services/planning.rs && ! grep -rqi "Override" apps/desktop/src/screens/plan crates/locus-core/src/services/planning.rs` |
| 34 | Assert every Plan stage source and core enum names only the canonical seven stages | 1,4 | `pnpm -C apps/desktop test -- plan/canonical-stages` |
