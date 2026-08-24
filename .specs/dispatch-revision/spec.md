# dispatch-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision`, `setup-revision` · **Blocks** M6 automation

## Purpose

The screen contract for Dispatch's three tabs — Autorun, Schedules, Runs — and for Settings →
Guardrails, reconciled against what `crates/locus-core/src/runtime/dispatch.rs` and
`crates/locus-core/src/store/dispatch.rs` already ship. The engine already has a durable queue,
parallelism caps, a priority policy, boundary-only preemption, and Stop all with a ten-minute restore
window. This spec is what closes the gap between that engine and the mockup: a tri-state autorun
switch with auto-suspension, the review-slot model that throttles autorun to what gets reviewed, the
five fixed exclusions, a Start-work builder for schedules with per-schedule guardrail overrides, the
full Runs verify vocabulary, and a persisted Settings → Guardrails surface. It changes no visual
design — `design-revision` already settled the rail, the locators, and the tokens.

## Governed by

- `PLAN.md` §Decisions — "desktop desktop revision" item 2, **Dispatch is a durable queue**
- `PLAN.md` §Workflow guardrails — the seven-guardrail table and defaults
- `PLAN.md` §M6 — Automation and discoverability — schedules, overlap skipped never queued
- `docs/UI_MOCKUP_REVIEW.md` — `## Dispatch` (Autorun, Schedules, Runs) and `## Settings` (Guardrails)
- `crates/locus-core/src/runtime/dispatch.rs`, `crates/locus-core/src/store/dispatch.rs` — the durable
  queue engine this spec's surface drives

## Contract

### Autorun

**On/off is a per-project switch, nothing else.** "On means agents in that project pick up their own
work and run it without you starting anything. Off means every run begins with you, or with a
schedule you wrote. There is no third setting and no per-task exception." The existing
`AutorunState` in `runtime/dispatch.rs` is a bare bool; it becomes a tri-state — **on**, **off**,
**suspended** — because suspension must be distinguishable from a manual off to resume on its own.

The "All projects" master reads **All on** / **All off** / **Mixed** from the per-project states, and
an eligible count excludes archived and suspended projects from the "on" tally. **Archived projects
are locked off** — the switch is disabled and reads "autorun cannot be turned on for an archived
project." `core.projects` carries no archival state today; this spec adds the minimal `archived_at`
column dispatch needs to read it, not a full archival feature.

**Auto-suspension.** A project's rolling verify pass rate is computed from its recent runs. Falling
under **60%** suspends autorun with "Verify pass fell to N%, under the 60% floor — it comes back on
its own when the number recovers." Recovering past 60% resumes it without a human action. This is
distinct from the project being turned off by a person, and the UI must say which happened.

**The review-slot model.** "A slot is one change you have not reviewed yet, not one agent. The median
developer reviews four changes a week; eight concurrent agents produce thirty-one. Autorun drains at
the rate you absorb, or it is just a way of generating a backlog faster." Per project:

- **Review debt** — landed, unread artifacts, oldest first.
- **Pauses at** — the debt threshold that pauses autorun for that project.
- **Inbox budget** — autorun-originated runs entering the queue, capped per hour.
- **Change ceiling** — lines and files, falling through to Settings → Guardrails' change-size ceiling
  when unset for the project.

**Never autoruns** — five fixed exclusions, true even when the project is on, each enforced at the
point an autorun-originated run would enter the queue:

1. Anything touching `migrations/**` — "A migration is append-only and irreversible in practice."
2. Any workflow containing a Gate node — "The gate is the point. Skipping it would be deleting it."
3. Anything over the change ceiling — "Past a reviewer's capacity, review degrades from semantic to
   syntactic."
4. A project under the 60% verify floor — the same computation as auto-suspension.
5. The first task of any plan — "You see what a plan produces once before it produces unattended."

### Stop all

The confirm dialog names exact scope before acting — running agents killed at the next iteration
boundary, autorun switched off in n projects, n schedules skipped not queued, branches/artifacts/memory
untouched — which `StopAllSnapshot` and `Store::stop_all` already capture and persist. New: the
**handoff toggle**. On, "Up to 30 seconds each. A successor starts from the payload instead of
re-deriving it" — `stop_all` writes a handoff per active run before marking it stopped. Off,
"Immediate. Work in flight is discarded and the next agent starts from the transcript" — no handoff
row, no delay. Either way the action is reversible for ten minutes and `Store::restore_stop_all`
already re-arms autorun and schedules exactly as they were; a banner afterward reports what stopped and
offers **Restore previous state**.

### Schedules

Header meta: schedule count, fired, skipped, next firing and timezone, **New schedule**.

**Start-work builder** — new; today's screen only renders a static cron readout with no builder.
*What runs* is **Project** ("Runs every active agent in the project on whatever it is already set to
work on… the agents' own assignments decide"; an agent with nothing assigned is skipped) or **Custom**
(agent, harness, project, an optional spec, an optional prompt — "A spec sets the contract, a prompt
narrows what to do with it — give it either, or both." Prompt-only: "A prompt produces a run and an
artifact, but no board task — nothing reaches the board without a plan."). `workflows.schedules`
required a `workflow_def_id`; this spec makes it nullable and adds the columns Custom mode needs
(`run_mode`, `agent_def_id`, `harness`, `project_id`, `spec_id`, `prompt`).

*Guardrails* is an optional per-schedule override — preset, max iterations, change ceiling, files
touched, network tier, token budget, plus resolved permission pills. "Anything left unset falls
through to Settings → Guardrails for #project. A ceiling reached here stops the run and splits it; it
does not fail." — new table, new breach behavior for schedule-originated runs.

*When* is **Run once, now** / **On a schedule** / **Hold** ("Schedules are yours. Autorun is the other
path, and it is a per-project switch, not something you attach work to."). A cron expression carries a
human readout and four presets (Hourly, Nightly, Weekdays 09:00, Once at a time I pick), or attaches to
an existing schedule. **Overlap is skipped, never queued** — unchanged from `.specs/schedules`.

A **misconfiguration banner** fires when a schedule skips most of its recent firings, with **Widen the
interval**: "A schedule that skips every firing is misconfigured, which is why the skips are a number
and not a silence." Schedule cards show live/paused state, cron and readout, workflow and step, last
result, skipped count, and a duration sparkline. The executions table —
`Fired · Schedule · Result · Duration · Evidence` — is recorded with a verify result, "green or red,
never merely 'finished'," which `workflows.executions` already enforces via its `status` check.

### Runs

The flat ledger: search, sort, date range, and three KPIs — spec-gap rate, noise reclassified, tokens
per passing run — which the metrics spec already computes as queries over existing columns;
this surface composes them rather than adding a metric. Table columns:
`When · Harness · Project · repo · Agent · role · Model resolved · Events · Errors · Tokens · Verify ·
Id`. The verify vocabulary is `running`, `passed`, `failed`, `failed ×n`, `waiting: gate`, `n/a`,
`aborted` — today's frontend `RunStatus` type only has four of these. `waiting: gate` reads the
`waiting` reason column `.specs/guardrails` already defines; `failed ×n` counts consecutive failed
iterations; `n/a` is a run with no verify command configured. Subtitle: "Every run, scheduled or not ·
a schedule is just one way a run starts."

### Settings → Guardrails

Sections: **Guardrails** (populated), Inbox & notifications, Harnesses, Repositories, Store,
Appearance, Account — the fixture screen (`GuardrailsView.tsx`) already renders all four groups from
`docs/UI_MOCKUP_REVIEW.md`'s table. What is new is persistence: **Parallelism**'s five controls map
onto the existing `DispatchPolicy` (`global_parallelism`, `per_project_parallelism`, `priority_method`,
`tie_break`, `preemption_enabled`) and `Store::set_dispatch_policy`; **Stopping conditions**,
**Change size**, and **Permissions** have no store today and get one row,
`core.guardrail_defaults`, mirroring the seven `.specs/guardrails` defaults plus the change-size and
permissions controls this screen adds.

"Defaults for every new run. A run can be given tighter limits than these; it can never be given looser
ones without an explicit override that is recorded on the run" — save validates every changed value is
tighter, or requires an explicit recorded override for a looser one. Footer: **Save defaults** /
**Reset to shipped values** — "Changes apply to runs started after saving. Nothing in flight is
retuned underneath itself," which the effective-value-per-run recording from `.specs/guardrails`
already guarantees; this spec only needs the save path to not touch a running run's recorded values.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `schedules` | this spec takes the screen contract, the Start-work builder, per-schedule guardrail overrides, and the misconfiguration surface; cron → workflow firing, overlap-skipped-never-queued, and execution recording in `.specs/schedules` stand |
| `guardrails` | this spec takes the Settings → Guardrails screen, the tighter-never-looser save rule, and per-schedule override fallthrough; the seven engine defaults and their enforcement (`max_iterations`, reflection, kill-and-reassign, waiting ≠ idle, idle detection, wall-clock ceiling, token budget) in `.specs/guardrails` stand |

## Acceptance

1. The "All projects" master reads All on / All off / Mixed from per-project state; archived and
   suspended projects never count toward "on" in the eligible count.
2. An archived project's switch is disabled and cannot be turned on.
3. A project whose rolling verify pass rate falls under 60% suspends automatically, distinguishably
   from a manual off, and resumes on its own once the rate recovers.
4. The review-slot gauge shows debt against Pauses at; autorun for that project pauses once debt
   reaches the threshold.
5. The inbox budget caps autorun-originated runs entering the queue to n per hour.
6. All five Never-autoruns exclusions block an autorun-originated enqueue even when the project is on:
   `migrations/**`, a Gate node in the workflow, over the change ceiling, under the verify floor, and
   the first task of any plan — each asserted independently.
7. Stop all's confirm dialog names exact scope — agent count, project count, schedule count, and that
   branches/artifacts/memory are untouched — before anything stops.
8. The handoff toggle on writes a handoff per active run before it stops; off discards immediately
   with no handoff row.
9. Stop all is restorable for ten minutes; restoring returns queued and running runs to queued and
   re-arms autorun and schedules exactly as they were.
10. A Project-mode schedule runs every active agent's own assignment and skips an agent with nothing
    assigned.
11. A Custom-mode schedule given only a prompt produces a run and an artifact, never a board task.
12. An unset per-schedule guardrail falls through to Settings → Guardrails; a ceiling reached on a
    schedule-originated run stops and splits it rather than failing it.
13. `When = Hold` creates a schedule that fires nothing until armed.
14. Each of the four cron presets resolves to a correct expression.
15. A schedule that skips most of its recent firings is flagged, and Widen the interval clears the
    flag by widening the cron interval.
16. The Runs verify column renders the full vocabulary: `running`, `passed`, `failed`, `failed ×n`,
    `waiting: gate`, `n/a`, `aborted`.
17. Settings → Guardrails accepts any tighter saved value and rejects a looser one without an explicit
    recorded override.
18. A saved guardrail default does not retune a run already in flight.

## Open

- **The verify-pass-rate window.** "Falls to 44%" implies a rolling measure but PLAN.md and
  `.specs/guardrails` name no window size or run count. This spec assumes the same window as agent
  trust's "last 20 runs" from the metrics spec until decided otherwise.
- **Project archival is out of scope here.** This spec adds only the `archived_at` column dispatch
  needs to lock autorun off; a full archive/unarchive feature (who can archive, what else locks) is
  not this spec's to design.
- **The handoff payload itself is `.specs/handoffs` (M3) scope**, which is not yet implemented in
  `locus-core`. Stop all's on-toggle path is specified against that payload's eventual contract; until
  it lands, the write path is testable only against a stub.
- **Change ceiling: one setting or two.** The mockup shows a per-project change ceiling on the Autorun
  tab and a global one in Settings → Guardrails, with the former said to "fall through" to the latter.
  Whether the per-project value is stored at all when unset, or is purely a display of the resolved
  global value, is left to implementation.
