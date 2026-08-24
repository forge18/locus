# review-qa — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `CheckSource` descriptor — name, tool-attribution label, kind, adapter — registered, not matched on; assert no `match` on a check-source name in `crates/locus-core` | — | `cargo test -p locus-core qa::check_source_is_data` |
| 2 | `Finding` shape: severity (`fail`\|`warn`, always exactly one), title, project, location, one-line explanation, check-source id, run id | 1 | `cargo test -p locus-core qa::finding_shape` |
| 3 | `CheckRun`: project, check-source, trigger (manual\|push\|hourly\|daily), started/finished timestamps | 1 | `cargo test -p locus-core qa::check_run_shape` |
| 4 | `qa_check_runs` and `qa_findings` migration | 3 | `cargo test -p locus-core store::qa_schema` |
| 5 | A new run's findings atomically replace its check source's previous result set | 2,4 | `cargo test -p locus-core qa::run_replaces_previous` |
| 6 | Per-project schedule setting persists Manual/Push/Hourly/Daily, defaulting to Manual | 3 | `cargo test -p locus-core qa::schedule_setting` |
| 7 | Manual Refresh and a scheduled firing call the same run entry point | 6 | `cargo test -p locus-core qa::triggers_share_entry_point` |
| 8 | Hourly/Daily firing while the source's check is still running is recorded as skipped, never queued | 7 | `cargo test -p locus-core qa::overlap_is_skipped` |
| 9 | Unit tests adapter maps `cargo nextest`/`vitest` results into findings with no `warn` severity | 2 | `cargo test -p locus-core qa::unit_tests_adapter` |
| 10 | Linters adapter maps `lint::LintReport` results one-for-one into findings | 2 | `cargo test -p locus-core qa::linters_adapter` |
| 11 | LSP diagnostics adapter maps `locus lsp diagnostics` output into findings | 2 | `cargo test -p locus-core qa::lsp_adapter` |
| 12 | An `unsupported` LSP verb becomes a `warn` finding, never an empty passing result | 11 | `cargo test -p locus-core qa::lsp_unsupported_not_empty` |
| 13 | Agent reviews adapter calls `pr::self_review`, tagged with the reviewing agent id and its custom prompt, sharing the function rather than reimplementing it | 2 | `cargo test -p locus-core qa::agent_review_adapter` |
| 14 | Send to Inbox creates an inbox item whose locator resolves to the finding | 2 | `cargo test -p locus-core qa::send_to_inbox_creates_item` |
| 15 | Sending to Inbox does not remove or mutate the finding row | 14 | `cargo test -p locus-core qa::finding_stays_listed` |
| 16 | Resolving the inbox item does not clear the finding; only a later run without it does | 15 | `cargo test -p locus-core qa::resolve_does_not_clear_finding` |
| 17 | `<QAView>` header with project name and last-run relative time | — | `pnpm -C apps/desktop test -- qa/header` |
| 18 | Schedule control — segmented Manual/Push/Hourly/Daily plus Refresh | 17 | `pnpm -C apps/desktop test -- qa/schedule-control` |
| 19 | Four group cards with icon, tool attribution, pass/fail/warn summary | 17 | `pnpm -C apps/desktop test -- qa/groups` |
| 20 | Finding row: severity, title, project · location, one-line explanation | 19 | `pnpm -C apps/desktop test -- qa/finding-row` |
| 21 | Send to Inbox / Sent to Inbox toggle per finding, wired to core | 20,14 | `pnpm -C apps/desktop test -- qa/send-to-inbox` |
| 22 | Footer text rendered verbatim | 17 | `pnpm -C apps/desktop test -- qa/footer` |
| 23 | QA reloads all four groups on project switch; assert Analytics is unaffected | 17 | `pnpm -C apps/desktop test -- qa/follows-project` |
