# agent-definitions — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Frontmatter schema type with serde and the four enums | — | `cargo test -p locus-core agents::frontmatter_parses` |
| 2 | Reject an invalid `model_tier`; default `task_class` to `code` | 1 | `cargo test -p locus-core agents::enum_validation` |
| 3 | Warn on an unknown key rather than failing the parse | 1 | `cargo test -p locus-core agents::unknown_key_warns` |
| 4 | Persist to `agents.agent_defs` as frontmatter JSONB plus a body text column | 1 | `cargo test -p locus-core agents::persists` |
| 5 | Versioning: save creates a new version, prior versions stay readable | 4 | `cargo test -p locus-core agents::save_creates_version` |
| 6 | A run pins its definition version | 5 | `cargo test -p locus-core agents::run_pins_version` |
| 7 | Editing mid-run does not affect the running run | 6 | `cargo test -p locus-core agents::immutable_once_referenced` |
| 8 | Validate `tools` against the marketplace index at save | 4 | `cargo test -p locus-core agents::tools_must_resolve` |
| 9 | Reject a cross-project `memory.scope` | 1 | `cargo test -p locus-core agents::memory_scope_never_cross_project` |
| 10 | Export to `.md` | 4 | `cargo test -p locus-core agents::export_md` |
| 11 | Import round-trips to an identical definition | 10 | `cargo test -p locus-core agents::import_export_roundtrip` |
| 12 | Materialize a definition into all twelve harness layouts | 4 | `cargo test -p locus-core agents::materializes_everywhere` |
| 13 | Nesting bounds: depth 3, fan-out 4, enforced in core | 4 | `cargo test -p locus-core agents::nesting_bounds` |
| 14 | A workflow may lower the bounds and never raise them | 13 | `cargo test -p locus-core agents::workflow_narrows_only` |
| 15 | Seed the six definitions the UI draws: builder, reviewer, interviewer, researcher, auditor, keeper | 4 | `cargo test -p locus-core agents::seeded_six` |
| 16 | Wire the Workshop agent-definitions screen to real definitions over IPC | 15 | `pnpm -C apps/desktop test -- agentdefs/from-core` |
