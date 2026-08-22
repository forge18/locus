# agent-session-research — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Query `finding` artifacts as a session-scoped research feed with seed/run/close provenance | `artifacts` | `cargo test -p locus-core artifact::session_research_feed` |
| 2 | Seed a child task session's research feed from its planning-session findings without promoting them | 1 | `cargo test -p locus-core artifact::research_inherits_from_plan` |
| 3 | Hand planning findings to the task session as research-feed seeds without auto-promoting them | `planning-module` | `cargo test -p locus-core planning::findings_seed_task_session` |
| 4 | At session close, promote only human-reviewed research findings into long-term memory | `memory`,1 | `cargo test -p locus-core memory::promotes_reviewed_session_findings` |
