# analytics-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Define explicit Analytics scope (`all` or project id) and apply it to every projection | — | `cargo test -p locus-core analytics::scope_applies_to_all_projections` |
| 2 | Resolve the five ranges into shared bucket series | — | `cargo test -p locus-core analytics::range_buckets` |
| 3 | Join range and scope once for every overview query | 1,2 | `cargo test -p locus-core analytics::range_and_scope_are_shared` |
| 4 | Project the four stat-card totals and selectable measure | 3 | `cargo test -p locus-core analytics::stat_cards_and_selected_measure` |
| 5 | Project trend data by the selected measure | 4 | `cargo test -p locus-core analytics::trend_tracks_selected_measure` |
| 6 | Project Model, Harness, Agent, Role, and Workflow breakdown rows | 3 | `cargo test -p locus-core analytics::breakdown_dimensions` |
| 7 | Derive task outcomes, cost by role, and expensive-to-land from board state and evidence | 3 | `cargo test -p locus-core analytics::task_outcomes_and_cost` |
| 8 | Add Landed after rework only to the project-scoped task projection | 1,7 | `cargo test -p locus-core analytics::project_rework_outcome` |
| 9 | Project workflow run median, p90, iterations, and verified count from one run set | 3 | `cargo test -p locus-core analytics::workflow_duration_projection` |
| 10 | Record retrieval feedback needed for useful and changed-answer percentages | — | `cargo test -p locus-core analytics::retrieval_feedback` |
| 11 | Project memory retrieval tiers, chips, and most-read list without fabricating unknown feedback | 3,10 | `cargo test -p locus-core analytics::memory_retrieval_projection` |
| 12 | Project extension usage only from materialized or invoked extension records | 3 | `cargo test -p locus-core analytics::extension_usage_projection` |
| 13 | Register the closed Telemetry facet set without capture source | — | `cargo test -p locus-core analytics::telemetry_facets_are_acp_only` |
| 14 | Compose event-log scope, range, search, and facets by intersection | 1,2,13 | `cargo test -p locus-core analytics::telemetry_query_intersection` |
| 15 | Count facets from the filtered result set rather than the corpus | 14 | `cargo test -p locus-core analytics::facet_counts_follow_result_set` |
| 16 | Project Sessions, Events, Tool errors, Output tokens, and session sparkline | 14 | `cargo test -p locus-core analytics::telemetry_stat_projection` |
| 17 | Project canonical action counts without synthesizing unavailable verbs | 14 | `cargo test -p locus-core analytics::action_vocabulary_projection` |
| 18 | Mark nonzero permission requests as an alarm | 17 | `cargo test -p locus-core analytics::permission_request_is_alarm` |
| 19 | Project allowlisted tool payload and anomaly notes | 14 | `cargo test -p locus-core analytics::tool_projection` |
| 20 | Project the Sessions table and its closed status vocabulary | 14 | `cargo test -p locus-core analytics::session_table_projection` |
| 21 | Render global Analytics with its All-projects banner and range tabs | 1,2 | `pnpm -C apps/desktop test -- analytics/global-header-and-range` |
| 22 | Render stat cards, selected-measure trend, and five-dimension breakdown | 4,5,6,21 | `pnpm -C apps/desktop test -- analytics/overview-metrics` |
| 23 | Render task outcomes, cost-by-role, expensive-to-land, and workflow durations | 7,8,9,21 | `pnpm -C apps/desktop test -- analytics/tasks-and-workflows` |
| 24 | Render memory retrieval and extension usage sections with filters | 11,12,21 | `pnpm -C apps/desktop test -- analytics/memory-and-extensions` |
| 25 | Render Setup → Analytics through the same component with project scope | 1,21 | `pnpm -C apps/desktop test -- setup/analytics-scoped` |
| 26 | Render the Overview / Telemetry sub-tab and 264px facet rail | 13,21 | `pnpm -C apps/desktop test -- analytics/telemetry-layout` |
| 27 | Render BM25 search, removable filter chips, Reset filters, and result-set facet counts | 14,15,26 | `pnpm -C apps/desktop test -- analytics/telemetry-filters` |
| 28 | Render Telemetry stat cards and canonical Actions alarm behavior | 16,17,18,26 | `pnpm -C apps/desktop test -- analytics/telemetry-actions` |
| 29 | Render Tools payload and Sessions table with all status variants | 19,20,26 | `pnpm -C apps/desktop test -- analytics/telemetry-tools-and-sessions` |
| 30 | Repoint `dashboard-metrics/spec.md` to the new screen contract | — | `grep -q "Superseded by.*analytics-revision" .specs/dashboard-metrics/spec.md` |
| 31 | Repoint `screens-dashboard/spec.md` to Analytics | 30 | `grep -q "Superseded by.*analytics-revision" .specs/screens-dashboard/spec.md` |
| 32 | Repoint the telemetry scope in `desktop-knowledge-review/spec.md` | 30 | `grep -q "analytics-revision" .specs/desktop-knowledge-review/spec.md` |
| 33 | Assert retired Dashboard has no rail item or route | 21 | `pnpm -C apps/desktop test -- nav/no-dashboard-route` |
| 34 | Assert global and project Analytics route through the shared locator resolver | 21,25,26 | `pnpm -C apps/desktop test -- nav/analytics-locators` |
