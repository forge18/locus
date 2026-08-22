# agent-session-controls — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Project `plan_update` into the session's one active plan without widening the canonical event vocabulary | `acp-client` | `cargo test -p locus-core acp::plan_update` |
| 2 | Implement restricted-schema elicitation with accept, decline, cancel, validation, and defaults | `acp-client` | `cargo test -p locus-core acp::elicitation` |
| 3 | Expose new-session, compact, clear-context, and context-view session commands | `acp-client` | `cargo test -p locus-core acp::session_commands` |
| 4 | Queue a steering prompt for the next turn boundary and cancel only the active turn | `acp-client` | `cargo test -p locus-core acp::steering_boundary` |
| 5 | Create a panel-requested subagent through the existing bounded invocation path | `acp-client` | `cargo test -p locus-core acp::panel_subagent` |
| 6 | Persist the dispatch-selected `bypass` or `gated` permission posture on the run | `run-supervisor` | `cargo test -p locus-core run::permission_posture` |
| 7 | A gated permission request waits for a human response; a bypass request raises the existing alarm | 6 | `cargo test -p locus-core run::permission_request_by_posture` |
| 8 | Snapshot a checkpoint before an edit; restore and undo preserve the transcript | `run-supervisor` | `cargo test -p locus-core run::checkpoints` |
| 9 | Replay a session into a newly attached Agent Pane without re-running the agent | `run-supervisor` | `cargo test -p locus-core session::panel_replay` |
| 10 | A gated-run `permission_request` becomes a human-action request rather than an alarm | `telemetry`,6 | `cargo test -p locus-core telemetry::permission_request_gated` |
| 11 | Preserve the request identity and diff payload needed to resolve a gate after replay | 10 | `cargo test -p locus-core telemetry::permission_request_replay` |
