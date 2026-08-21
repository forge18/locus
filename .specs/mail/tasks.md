# mail — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `mail` schema: threads, messages, delivery state | — | `cargo test -p locus-core mail::schema` |
| 2 | `locus mail send` between two agents | 1 | `cargo test -p locus-cli mail::send` |
| 3 | `locus mail list` and `read` | 2 | `cargo test -p locus-cli mail::list_read` |
| 4 | `locus mail reply` threading correctly | 3 | `cargo test -p locus-cli mail::reply_threads` |
| 5 | `locus mail drain` returning all pending and clearing | 3 | `cargo test -p locus-cli mail::drain` |
| 6 | `locus mail wait` with a 15-minute default timeout | 3 | `cargo test -p locus-cli mail::wait_times_out` |
| 7 | `wait` sets the run's `waiting` state with a reason | 6 | `cargo test -p locus-core mail::wait_sets_waiting` |
| 8 | Assert the idle guardrail does not fire while `waiting` is set | 7 | `cargo test -p locus-core mail::waiting_suppresses_idle` |
| 9 | The shared waiting mechanism with its four callers | 7 | `cargo test -p locus-core waiting::four_callers` |
| 10 | Human inbox as an addressee in the same system | 1 | `cargo test -p locus-core inbox::human_is_a_participant` |
| 11 | `locus ask` reaching the inbox with its session, and blocking | 10 | `cargo test -p locus-cli ask::blocks_and_reaches_inbox` |
| 12 | Every inbox item carries a resolvable locator | 10 | `cargo test -p locus-core inbox::items_resolve` |
| 13 | Reject an item with no resolvable target as a notification | 12 | `cargo test -p locus-core inbox::notifications_are_not_inbox_work` |
| 14 | Assert a normally running session produces zero inbox items | 10 | `cargo test -p locus-core inbox::silence_is_the_default` |
| 15 | Mail survives a harness swap mid-project | 2 | `cargo test -p locus-core mail::survives_harness_swap` |
| 16 | Wire the Inbox screen to real items over IPC | 12 | `pnpm -C apps/desktop test -- inbox/from-core` |
