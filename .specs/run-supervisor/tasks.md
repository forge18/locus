# run-supervisor — tasks

**Spike 1 can void tasks 5-6** — how a run authenticates at start depends on the mechanism it settles.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Session, run and turn types and their tables | — | `cargo test -p locus-core session::model` |
| 2 | Session holds agent@version, branch, task, memory base, pane state | 1 | `cargo test -p locus-core session::holds` |
| 3 | Run holds events, usage, exit status, artifacts, resolved model id | 1 | `cargo test -p locus-core run::holds` |
| 4 | Spawn: materialize config, build or reuse the image, start the container, open the ACP session | 3 | `cargo test -p locus-core run::spawns` |
| 5 | Inject credentials at run start per the Spike 1 mechanism | 4 | `cargo test -p locus-core run::authenticates` |
| 6 | Assert nothing long-lived persists in the container after start | 5 | `cargo test -p locus-core run::no_persisted_secret` |
| 7 | Stream: normalized ACP events to the UI over `Channel<Event>` | 4 | `cargo test -p locus-core run::streams_acp` |
| 8 | Normalize: hand the ACP stream to the telemetry adapter for this source | 4 | `cargo test -p locus-core run::normalizes` |
| 9 | Persist every normalized event with `run_id`, `seq`, `ts`, `raw` | 8 | `cargo test -p locus-core run::persists_events` |
| 10 | Cancel, recording the reason | 4 | `cargo test -p locus-core run::cancels` |
| 11 | Pause: finish the current turn, hold before the next, keep the container up | 4 | `cargo test -p locus-core run::pause_holds_not_freezes` |
| 12 | Second run in the same session inherits branch, task and memory base | 2,4 | `cargo test -p locus-core session::survives_reset` |
| 13 | Resume primes a new run from the session's own events | 12 | `cargo test -p locus-core session::resume_from_events` |
| 14 | Resume works on a harness with no native session id | 13 | `cargo test -p locus-core session::resume_without_native_id` |
| 15 | Store a native session id on the run where the harness gives one | 13 | `cargo test -p locus-core run::native_session_id` |
| 16 | Boot reconciliation: alive container re-attaches | 4 | `cargo test -p locus-core run::reattach_on_boot` |
| 17 | Boot reconciliation: gone container closes as aborted and emits | 16 | `cargo test -p locus-core run::aborts_orphans` |
| 18 | An aborted-on-boot run files an inbox item | 17 | `cargo test -p locus-core run::abort_reaches_inbox` |
| 19 | `locusd` as a background service outliving the window | 4 | `cargo test -p locus-core daemon::outlives_window` |
| 20 | No terminal on the agent path; an agent run renders as events only | 7 | `cargo test -p locus-core run::no_terminal_surface` |
| 21 | Two harnesses running concurrently, events indistinguishable downstream | 9 | `cargo test -p locus-core run::two_harnesses_concurrent` |
| 22 | A third run on a different project appears in the same strip | 21 | `pnpm -C apps/desktop test -- strip/cross-project` |
| 23 | Persist the dispatch-selected `bypass` or `gated` permission posture on the run | 4 | `cargo test -p locus-core run::permission_posture` |
| 24 | A gated permission request waits for a human response; a bypass request raises the existing alarm | 23 | `cargo test -p locus-core run::permission_request_by_posture` |
| 25 | Snapshot a checkpoint before an edit; restore and undo preserve the transcript | 4 | `cargo test -p locus-core run::checkpoints` |
| 26 | Replay a session into a newly attached Agent Pane without re-running the agent | 9,19 | `cargo test -p locus-core session::panel_replay` |
