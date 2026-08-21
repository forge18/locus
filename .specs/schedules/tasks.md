# schedules — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `workflows.schedules` table with a cron expression | — | `cargo test -p locus-core sched::table` |
| 2 | Cron parser and next-fire computation | 1 | `cargo test -p locus-core sched::cron_parses` |
| 3 | Timezone and DST handling, decided and tested | 2 | `cargo test -p locus-core sched::dst` |
| 4 | Scheduler firing a workflow in `locusd` | 2 | `cargo test -p locus-core sched::fires` |
| 5 | Fires with the app window closed | 4 | `cargo test -p locus-core sched::fires_headless` |
| 6 | Record the execution with its verify result | 4 | `cargo test -p locus-core sched::records_verify_result` |
| 7 | Overlap detection against a running execution | 4 | `cargo test -p locus-core sched::detects_overlap` |
| 8 | Skip and drop on overlap | 7 | `cargo test -p locus-core sched::skips_never_queues` |
| 9 | Assert nothing queues — no backlog after a slow run | 8 | `cargo test -p locus-core sched::no_backlog` |
| 10 | Record skipped firings as a visible count | 8 | `cargo test -p locus-core sched::skips_are_counted` |
| 11 | Restart without losing or double-firing a schedule | 4 | `cargo test -p locus-core sched::restart_safe` |
| 12 | Pause and resume, keeping history | 1 | `cargo test -p locus-core sched::pause_resume` |
