# design-revision

**Milestone** M0.7 · **Depends on** M0.6 (`design-desktop`) · **Blocks** every other M0.7 feature and
every desktop surface built after it.

## Purpose

Adopt the current mockup — `docs/UI mockups for PLAN.md/Locus v2.dc.html` and `AgentPanel.dc.html`.
M0.6 was written from an earlier desktop iteration, with a different rail vocabulary and stage
pipeline, and it knows nothing of six surfaces the current design draws in full. This feature settles the vocabulary, the screen inventory, and the three decisions the
rest of M0.7 depends on. It changes no implementation on its own.

The screen-by-screen contract lives in [`docs/UI_MOCKUP_REVIEW.md`](../../docs/UI_MOCKUP_REVIEW.md);
this spec does not restate it.

## Governed by

- `PLAN.md` §Decisions, §Navigation, §The planning module, §The Workflow Canvas, §Credentials
- `docs/UI_MOCKUP_REVIEW.md` — the reviewed contract for all 30 views and the agent panel
- `docs/UI mockups for PLAN.md/Locus v2.dc.html`, `AgentPanel.dc.html` — visual reference, never
  production code

## Contract

### Authority

`Locus v2.dc.html` at the top of `docs/UI mockups for PLAN.md/` is the design. The earlier v2 handoff
and its README are superseded and must not be cited by any spec. `Locus UI mockups.html` is a bundle
of the same file. `Locus.dc.html` and the earlier desktop handoff are v1.

### Vocabulary

Rail categories are **Setup, Plan, Manage, Interact, Bots, Review** (project-scoped) and **Analytics,
Memory, Settings, Workshop** (cross-project). Inbox and Dispatch are title-bar pills, not rail
items. Bots is the persistent named-agent surface and carries only the `bots` view. Its list rows may
show the bot's derived avatar, but the screen remains the only avatar surface; no panel or route
contract changes. The former project,
task, metrics, and project-list labels are retired: no spec, fixture, route id, or component may
reintroduce them.

### Screen inventory

Thirty views, each with a `locus://` locator and exactly one rail category. Every view is
routable on its own; no two views may share a route. The inventory is:

`inbox`, `status` (Analytics), `telemetry`, `mail`, `projects` (Setup), `plan`, `sessions` (Manage),
`interact`, `bots` (Bots), `qa` (Review), `autorun`, `schedule`, `runs`, `short`, `memory`, `artifact`, `wiki`,
`settings`, and the twelve Workshop views. Workshop groups `cli`, `harnesses`, and `providers` under
Plugins; `agents`, `commands`, `hooks`, `linters`, `styles`, `rules`, `skills`, `canvas`, and the
Workflows list under Extensions.

### Decisions

1. **The plan pipeline is seven stages** — Inputs, Orient, Converse, Synthesis, Recommend,
   Decompose, Approved. Audit and Override are not stages. The auditor is an agent role that runs on
   a schedule; user override is expressed by editing during Recommend and Decompose. The confidence
   ratchet and the `open[n]` gap counters survive inside Recommend.
2. **ACP is the only capture source.** The mockup's Telemetry facet listing hooks, acp, stream-json,
   and session-log is stale fixture content. `.specs/acp-client` and `.specs/telemetry` stand.
3. **The plugin registry decides the harness roster.** The first-party roster contains Pi only; a
   trusted user harness plugin may add another entry. Counts rendered in any surface derive from the
   registry, never from a constant.

### Tokens

The v2 semantic tokens carry over unchanged from `theme-system`: `--ac` for human action and focus,
`--ac2` for machine activity, `--data-*` for magnitude, `--ok`/`--bad` for outcome. The accent is
never a chart bar or a broad fill. New surfaces in this milestone introduce no new colour roles.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `design-desktop` | this spec, where the two differ; `design-desktop` remains the record of what M0.6 built |

Every spec superseded by an M0.7 feature carries a pointer line to its replacement, so a superseded
spec cannot be mistaken for a current one.

## Acceptance

1. No file under `.specs/` or `PLAN.md` cites an earlier handoff directory as governing.
2. No M0.7 spec, fixture, route id, or component names a retired rail category; the M0.6 specs retain the historical labels they implemented.
3. Every one of the 30 views is named by exactly one governing spec and carries one locator.
4. Every spec this milestone supersedes carries a pointer line naming its replacement.
5. The plan pipeline is described as seven stages in `PLAN.md`, `.specs/plan-revision`, and
   `crates/locus-core`; `Audit` and `Override` appear only as the auditor-role note.
6. Harness and downgrade counts in prose are derived from the registry; no prose assumes a fixed
   first-party harness count or downgrade total.

## Open

- The autorouting bands include a `minimal` effort; Plan → Decompose cycles `low`, `medium`, `high`,
  `xhigh`. One effort vocabulary must win — resolved in `workshop-revision`.
