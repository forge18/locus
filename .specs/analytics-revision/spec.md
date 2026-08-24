# analytics-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision`, `telemetry` · **Blocks** M6
analytics implementation and the project Analytics tab in `setup-revision`

## Purpose

Define the current Analytics surface: a global overview that deliberately ignores the project selector,
a project-scoped instance in Setup, and an Analytics → Telemetry sub-tab over the normalized event log.
The mockup's old Dashboard contract does not cover range bucketing, the measure-selected projections,
task outcomes, p90 workflow duration, memory retrieval quality, extension usage, or the queryable event
ledger. This spec adds projections, not a second telemetry store: every number is derived from persisted
runs, events, artifacts, memory retrievals, and materialization records.

## Governed by

- `PLAN.md` §Telemetry — normalized event vocabulary and the rule that missing verbs are recorded, never synthesized
- `PLAN.md` §Navigation — Analytics is Cross-Project and Telemetry is its sub-tab
- `.specs/dashboard-metrics/spec.md` — source queries and definitions that remain valid
- `.specs/telemetry/spec.md` — event persistence and capture source
- `docs/UI_MOCKUP_REVIEW.md` — Analytics and Analytics → Telemetry

## Contract

### Scope and range

Global Analytics is the one surface that ignores the project selector: its subtitle is **All projects**.
Setup → Analytics uses the identical component and query shape but scopes every run, retrieval, task, and
extension record to that project; it adds the `Landed after rework` task outcome. The component takes an
explicit scope value (`all` or a project id), never infers scope from a route name.

Range tabs are **24h / 7d / 30d / 90d / All**. They resolve respectively to 24 hourly buckets, 7 daily
buckets, 30 daily buckets, 13 weekly buckets, 12 monthly buckets, and the full retained history using the
coarsest suitable bucket. Every card, chart, table, and list reads the same resolved range; a range change
cannot leave one panel stale.

### Overview

Four selectable stat cards — **Spend**, **Tokens**, **Cache read**, **Runs** — select the measure used by
the trend and breakdown bar. The cards themselves remain all four totals; selecting one redraws every
measure-dependent surface below without changing the range or scope.

The trend chart exposes **Spend / Tokens / Cache read**. The breakdown dimensions are **Model, Harness,
Agent, Role, Workflow**, with columns `<dimension> · Tokens · Cache · Spend · Runs · Per run`; its bar
tracks the selected measure, not a fixed metric.

The **Tasks** projection contains outcome bars (**Landed, Abandoned, Still open**, plus project-scoped
**Landed after rework**), a cost-by-role table (`Role · Landed · Cost · Runs · First try`), and an
expensive-to-land list with iterations and cost. The outcome is derived from board/task completion and
verify evidence, not a model assertion.

**Run times by workflow** contains `Workflow · Runs · Median · p90 · Iter · Verified` and a dual
median/p90 bar. Durations are wall-clock run duration; iterations and verified counts share the same run
set.

**Memory retrievals** groups Short-term, Long-term, Artifacts, and Wiki. Each tier has hits, useful
percentage, and average tokens; stat chips show recalls per run, recalls that changed the answer, facts
written, and promoted-to-long-term count. A Most read list uses the same range and scope.

**Extension usage** filters **all, skill, rule, hook, linter, style, agent** and lists extension name,
hit count, and a derived loading/failure note. It is usage of a materialized or invoked extension, not
merely the count of definitions.

### Telemetry sub-tab

Telemetry has a 264px facet rail and a query surface. Facets are **harness, project, agent · role, model
tier, verify, arbiter class, branch**. The mockup's capture-source facet is intentionally absent: ACP is
the only capture source. Facet counts are the current result set, never corpus totals. Active selections
become removable chips; **Reset filters** clears every facet and search term.

BM25 searches the normalized event log. Stat cards are Sessions, Events, Tool errors, Output tokens, and
a sessions-over-time sparkline. Search, facets, scope, and range compose by intersection; the result
projection is one query definition shared by cards, tables, and facet counts.

The **Actions** panel displays the canonical vocabulary and counts: `tool_call`, `tool_result`,
`assistant`, `thinking`, `user`, `tool_error`, `subagent_start`, `subagent_stop`, `session_start`,
`session_end`, `aborted`, `permission_request`. A nonzero `permission_request` is a misconfiguration
alarm; a source that cannot report a verb leaves it absent rather than inventing zero-valued events.

The **Tools** panel groups allowlisted tool payload by tool and flags anomalies. The sessions table is
`When · Harness · Project · repo · Agent · role · Model(s) · Runs · Events · Errors · Tokens · Status ·
Id`; status is one of `running`, `stuck n/3`, `waiting: gate`, `idle Nm`, `handed off`, `closed`,
`aborted`.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `dashboard-metrics` — screen contract | this spec's global and project overview plus Telemetry sub-tab; its existing event-derived metric definitions stand |
| `screens-dashboard` | this spec; Dashboard is retired in favor of Analytics |
| `desktop-knowledge-review` — telemetry viewer only | this spec's Analytics → Telemetry sub-tab |

Each replaced spec carries a pointer line to this spec, scoped as above.

## Acceptance

1. Global Analytics ignores the project selector; Setup → Analytics scopes every projection to its project.
2. A range change updates every overview and Telemetry projection from one resolved range.
3. The four stat cards remain visible while selecting one changes the trend and breakdown measure.
4. Breakdown supports exactly Model, Harness, Agent, Role, and Workflow with the stated columns.
5. Task outcomes derive from board state and evidence; project scope adds Landed after rework only.
6. Workflow duration presents median and p90 over the same scoped run set.
7. Memory retrieval tiers, usefulness, tokens, and most-read list use the same range and scope.
8. Extension usage includes materialized/invoked usage only and filters the seven stated kinds.
9. Telemetry has no capture-source facet; ACP is not rendered as one selectable source among stale alternatives.
10. Search, facets, range, and scope compose by intersection, and facet counts describe the resulting set.
11. Actions display the canonical vocabulary without synthesizing missing verbs.
12. A nonzero permission-request count renders as an alarm, not a success metric.
13. Tools derive from the allowlist and carry anomaly notes.
14. Sessions render the stated columns and closed status vocabulary.
15. `dashboard-metrics`, `screens-dashboard`, and the telemetry portion of `desktop-knowledge-review` carry scoped supersession pointers.

## Open

- The event-log retention period and BM25 index refresh policy are owned by `telemetry`; this feature consumes their result.
- `useful` and `changed the answer` need an explicit retrieval-feedback event. Until it exists, the UI renders unknown rather than a fabricated percentage.
- The retention-aware bucket chosen for **All** is an implementation detail, provided every panel shares it.
