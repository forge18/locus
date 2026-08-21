# planning-module — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Three agent definitions: interviewer, researcher, auditor, with their `task_class` | — | `cargo test -p locus-core planning::three_agents` |
| 2 | Run each in its own container over ACP | 1 | `cargo test -p locus-core planning::separate_containers` |
| 3 | Assert no shared context between them | 2 | `cargo test -p locus-core planning::no_shared_context` |
| 4 | Step 1 Inputs: goal, project, target repo, involved repos, optional tools and workflow | 1 | `cargo test -p locus-core planning::inputs` |
| 5 | Involved repos cloned read-only to `/context/<repo>` and indexed | 4 | `cargo test -p locus-core planning::context_repos` |
| 6 | Step 2 Orient: bounded, once — index, pull wiki and decisions, prior art, resolve the index | 5 | `cargo test -p locus-core planning::orient_is_bounded` |
| 7 | Step 3 Converse: the interviewer's question loop | 6 | `cargo test -p locus-core planning::question_loop` |
| 8 | Topics ranked by bearing on the goal, dropped when they do not | 7 | `cargo test -p locus-core planning::goal_ranks_topics` |
| 9 | Dispatch the researcher for fact, feasibility, prior art | 7 | `cargo test -p locus-core planning::dispatch_researcher` |
| 10 | Assert the researcher is never asked for intent | 9 | `cargo test -p locus-core planning::researcher_never_asked_intent` |
| 11 | Scope decision resolving inline, in both directions, requiring the human | 7 | `cargo test -p locus-core planning::scope_needs_human` |
| 12 | Count scope decisions separately from questions | 11 | `cargo test -p locus-core planning::scope_counted_separately` |
| 13 | Record a rejected increase as a `decision`; do not re-propose | 11 | `cargo test -p locus-core planning::rejection_is_remembered` |
| 14 | Step 4 pass 1: completeness — types, transitions, edge cases, trust boundaries, errors | 7 | `cargo test -p locus-core planning::completeness_pass` |
| 15 | Step 4 pass 2: reduction — cut the unsupported, rewrite the ambiguous | 14 | `cargo test -p locus-core planning::reduction_pass` |
| 16 | Assert the spec shrinks in pass 2 | 15 | `cargo test -p locus-core planning::reduction_subtracts` |
| 17 | LENS-style latent-requirement pass, always proposed and never adopted | 15 | `cargo test -p locus-core planning::latent_requirements_proposed` |
| 18 | Step 5 Audit: the 29148 rubric | 15 | `cargo test -p locus-core planning::audit_rubric` |
| 19 | The two-reader test: two agents restate, diff the restatements | 18 | `cargo test -p locus-core planning::two_reader_test` |
| 20 | Report divergence as the ambiguity | 19 | `cargo test -p locus-core planning::divergence_is_the_ambiguity` |
| 21 | Auditor checks the recommendation's goal restatement against what was set | 18 | `cargo test -p locus-core planning::goal_drift_check` |
| 22 | Loop back to step 3 at most once | 18 | `cargo test -p locus-core planning::audit_loops_once` |
| 23 | Residual findings become named weaknesses, not blockers | 22 | `cargo test -p locus-core planning::residual_is_a_weakness` |
| 24 | Step 6 Recommend: the recommendation object with all eight fields | 18 | `cargo test -p locus-core planning::recommendation_shape` |
| 25 | Confidence as a named condition, not a bare number | 24 | `cargo test -p locus-core planning::confidence_has_condition` |
| 26 | Effort ratchet on the five triggers, never de-escalating | 7 | `cargo test -p locus-core planning::ratchet` |
| 27 | Step 7 Override and step 8 Approve or Reject | 24 | `cargo test -p locus-core planning::approve_or_reject` |
| 28 | Approval lands tasks on the board; rejection keeps the draft | 27 | `cargo test -p locus-core planning::approval_lands_tasks` |
| 29 | Tasks ordered hardest first, as vertical slices | 28 | `cargo test -p locus-core planning::hardest_first` |
| 30 | Spec committed as a wiki page, contract separated from design | 28 | `cargo test -p locus-core planning::spec_is_a_wiki_page` |
| 31 | Workflow proposed, never committed | 28 | `cargo test -p locus-core planning::workflow_is_proposed` |
| 32 | W3C excerpt anchoring: quote selector plus position selector | 7 | `cargo test -p locus-core planning::excerpt_anchoring` |
| 33 | Flag a duplicate `exact` match for audit rather than anchoring | 32 | `cargo test -p locus-core planning::duplicate_excerpt_flagged` |
| 34 | Forward and backward traceability across the six hops | 32 | `cargo test -p locus-core planning::traceability_both_ways` |
| 35 | Re-planning: rewrite a not-started task in place | 28 | `cargo test -p locus-core planning::replan_not_started` |
| 36 | Re-planning: flag and notify for an in-progress task, never mutate | 35 | `cargo test -p locus-core planning::replan_in_progress` |
| 37 | Re-planning: emit a delta task for a Done one, never touch it | 35 | `cargo test -p locus-core planning::replan_done_emits_delta` |
| 38 | Re-planning: close a deleted requirement as `superseded` | 35 | `cargo test -p locus-core planning::replan_supersedes` |
| 39 | Specialization records as wiki `concept` pages, applied above a confidence threshold | 14 | `cargo test -p locus-core planning::specialization_records` |
| 40 | Wire the Plan screen to a real ACP conversation | 27 | `pnpm -C apps/desktop test -- plan/from-core` |
