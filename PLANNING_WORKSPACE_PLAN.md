# Planning Workspace and Navigation Plan

**Status:** Proposed. [`PLAN.md`](PLAN.md) remains authoritative until this proposal is accepted and decomposed into `.specs/` contracts.

## Reason for existence

Locus models planning as one project-scoped, seven-stage conversation producing one spec and task set. It must also plan a project, feature, or amendment; survive agent and application restarts; and always finish with reviewed specs and executable tasks. This document defines the target product model, navigation, capability policy, impact, and migration order. It authorizes no code.

## Settled decisions

1. Planning is a durable **Planning Workspace**, not one agent session.
2. Workspace scope is `amendment`, `feature`, or `project`.
3. Planning depth adapts to scope, uncertainty, and impact.
4. Every approved workspace ends in specs and tasks. A project spec map is intermediate.
5. Workspaces persist until explicitly approved or deleted.
6. Existing bounded spec planning remains the leaf mechanism.
7. Shell has no global project selector; pages and objects own scope.
8. Plan lists all workspaces. Creation requires an existing project or **New Project**.
9. Manage defaults to all projects. Review requires a project before loading QA.
10. Bots becomes **Workers**. Interact route is removed; unique capabilities must move or be explicitly retired.
11. Analytics Overview moves into Telemetry. Dispatch retains Autorun, Schedules, and Runs. Mail is a background project process.
12. Workshop contains Extensions, Plugins, Knowledge, and Settings. First three have sublinks.
13. Agent definitions can inherit or restrict CLI tools, commands, and skills independently.

## Product model

### One endpoint, different breadth

| Scope | Result |
| --- | --- |
| Amendment | Existing-spec amendment or delta spec → delta tasks |
| Feature | One or more bounded specs → all tasks |
| Project | Shared brief + coordinated spec set → all tasks across every spec |

A project workspace is complete only when every in-scope area has a reviewed spec and every requirement maps to a task or explicit non-task outcome.

### Adaptive spine

Every workspace satisfies four gates:

1. **Frame** — goal, subject, use cases, success.
2. **Understand** — relevant context inspected; material unknowns resolved or named.
3. **Shape** — scope-appropriate artifacts produced.
4. **Commit** — complete spec/task package reviewed and approved.

Evidence selects activities: capture bullets; define goals/use cases; orient repositories; research prior art; brainstorm alternatives; map architecture; assess blast radius; grill requirements; audit ambiguity; define milestones; build spec map; decompose specs; audit integration; approve.

UI explains added or omitted activities. Users can deepen, narrow, split, merge, or revisit work. Planning deepens for multiple goals/write repos, shared interfaces, migrations, security boundaries, new subsystems, missing prior art, contradictions, systemic impact, or major open decisions. It stays focused for bounded, familiar, single-repo work with clear acceptance and no shared-contract change.

### Project flow

1. Capture idea and functionality bullets.
2. Establish goal, use cases, and success criteria.
3. Brainstorm shape and alternatives.
4. Propose and confirm spec boundaries.
5. Grill every child spec at appropriate depth.
6. Audit interfaces, duplication, gaps, terminology, migrations, and security.
7. Decompose every spec into verified, dependency-linked tasks.
8. Merge child dependencies into one project task graph.
9. Approve exact workspace revision.
10. Materialize approved tasks or a selected execution tranche without changing the approved package.

Step 4's spec map is scaffolding, not output completion.

## Durable workspace contract

### Lifecycle

```text
Draft → In progress → Ready for approval → Approved
  └────────────────────────────────────────→ Deleted
```

Agent `active`, `idle`, and `working` presence is separate. Leaving the screen never changes lifecycle.

### Persisted state

- Bullets, goal, use cases, success criteria, scope, and depth
- Current understanding, decisions, rejected alternatives, constraints, risks, open questions
- Research findings and sources
- Shape and spec map
- Child specs, stable requirements, revisions, confidence conditions, open items
- Tasks, verification, routing, dependencies
- Agent-session links, activity history, pending decisions
- Staleness/impact markers and current/approved revision IDs

Transcript is provenance, not canonical resumable state. Reopening shows last checkpoint, active spec/activity, pending decisions, completed/failed background work, relevant changes, and recommended next action. Replacement agents receive a structured handoff.

Approval freezes an exact revision. Later edits create amendments/revisions. Shared-decision changes mark dependents potentially stale; they never silently rewrite approved, active, or completed work.

## Leaf behavior to preserve

Keep bounded planning behavior in [`crates/locus-core/src/services/planning.rs`](crates/locus-core/src/services/planning.rs):

- Interviewer/researcher/auditor separation
- Goal as input; human-approved scope changes
- Two-pass synthesis and unsupported-clause removal
- Stable requirement IDs and changed-requirement re-audit
- Two-reader ambiguity check and bounded audit loop
- Evidence-driven effort ratchet and hardest-first tasks
- Bidirectional traceability and replanning rules
- No board materialization before approval

Seven stages may remain a child-spec profile; they are not workspace-global progress.

## Information and approval model

Add a parent above bounded plans; do not make `core.plans` recursive:

```text
PlanningWorkspace
├── brief, use cases, decisions, risks, revisions
├── WorkspaceSpec[]
│   ├── target repository
│   ├── requirements/findings
│   ├── tasks/routing
│   └── planning sessions
└── cross-spec dependencies and materialization records
```

Required trace: workspace → project; use case → spec → requirement → task → board task → evidence; decision → affected requirements; session → workspace/spec; approved revision → board work. Each spec has one writable target repo. Existing plans backfill as one-spec workspaces. New schema uses a new migration.

Planning state is event-backed and projected. Autosave/background agents use optimistic revision checks. Final approval is idempotent and transactional: freeze revision; verify specs and coverage; validate DAG; reject cycles/unowned integration; create board events/tasks/edges; record provenance; prevent duplicate retries.

## Navigation

```text
Projects
Workers
Telemetry

──────────────
AUTOMATION
  Plan
  Manage
  Review

──────────────
WORKSHOP
  Extensions
    Agents
    Commands
    Context
    Hooks
    Linters
    Output styles
    Rules
    Skills
    Workflows
  Plugins
    CLI Tools
    Harnesses
    Providers
  Knowledge
    Short-term
    Long-term
    Artifacts
    Wiki
  Settings
```

Automation and Workshop are structural headers, not shared data scopes or landing pages.

### Page scope

| Surface | Default |
| --- | --- |
| Projects | All projects |
| Workers | All; creation requires project |
| Telemetry | All; includes Analytics Overview |
| Plan | All workspaces; creation requires existing/new project |
| Manage | All; local selector defaults All |
| Review | None; asks before project QA |
| Extensions | App-wide; Context selects project internally |
| Plugins | App-wide |
| Knowledge | All with local selector |
| Settings | App-wide |
| Dispatch | All with own filters |

No page inherits hidden shell scope. Page filters are independent. Dispatch owns Autorun/Schedules/Runs through title-bar pill, palette, and deep links. Mail surfaces through project history, Workers, tasks/workflows, Inbox for human action, and Telemetry diagnostics.

List routes may use `all`; object routes remain project-owned. Route definitions must support multiple allowed scopes. Production route authority moves out of `apps/desktop/src/fixtures/desktop-screen-inventory.ts`; fixtures derive from production routes.

## Workers and Interact

Workers owns persistent named agents, conversations, branches, routines, and warm-window behavior. Interact route is removed only after deciding where these capabilities go: disposable board-less sessions, changed-file inspection, commit, promote, discard, and session research. No capability disappears because screens look similar.

## Agent capability policy

Workshop → Extensions → Agents controls CLI Tools, Commands, and Skills independently:

```text
CapabilityPolicy<T>
  DeferToProject
  AllowOnly(Set<T>)
```

Effective set = project policy ∩ agent restriction ∩ workflow/task restriction. Agents narrow, never expand. `AllowOnly([])` means none. New agents default to project. Existing explicit tools/skills migrate to `AllowOnly`; empty lists never silently become inheritance. Commands are new.

Runs record agent version, project-policy revision, workflow/task restriction revision, resolved sets, and materialization/image digest. Project changes affect future runs only. Effective CLI tools remain the image/install set; commands and skills materialize only from the effective set.

## Impact

Primary code surfaces:

- `crates/locus-core/src/services/{planning,agents}.rs`
- `crates/locus-core/src/store/planning.rs`
- Planning/project/session/artifact/event-log/board projections and migrations
- `apps/desktop/src/{data/plan.ts,nav/,shell/,screens/plan/,screens/bots/,screens/interact/}`
- `apps/desktop/src/screens/workshop/ExtensionEditor.tsx`
- `apps/desktop/src-tauri/src/lib.rs`
- Project, Manage, Review, Telemetry, Knowledge, Dispatch providers

Contracts to revise/supersede: [`PLAN.md`](PLAN.md); `.specs/{plan-revision,planning-module,design-revision,shell-revision,interact-sessions,bots,manage-revision,review-qa,workshop-revision,agent-definitions,desktop-data-integration}/`; [`docs/UI_MOCKUP_REVIEW.md`](docs/UI_MOCKUP_REVIEW.md).

## TODO alignment

Root [`TODO.md`](TODO.md) changes implementation order:

- Remove global project switcher instead of wiring it.
- Settle this contract before Plan live-data integration.
- Replace fixture-backed Plan/routes with explicit live and demo providers.
- Fold planning into event log; add no direct projection writes.
- Resolve Dispatch/ACP end-to-end before background-planning promises.
- Replace Bots work with Workers after resolving Interact behavior.
- Move Analytics Overview into Telemetry and Memory/Wiki/Artifacts into Knowledge.
- Keep Autorun/Schedules/Runs under Dispatch; make Mail project-owned background behavior.

## Migration plan

1. **Adopt contracts** — approve direction; update `PLAN.md`, TODO, and superseding specs.
2. **Replace navigation authority** — production route manifest, new groups, page-owned scope, compatible deep links.
3. **Add workspace wrapper** — workspace/revision projections; link/backfill existing plans.
4. **Deliver persistence/resume** — session links, activities, decisions, IPC/channels; remove Plan fixtures.
5. **Build Planning Room** — Brief, Shape, Specs, Tasks, Coverage, Activity; child-stage detail.
6. **Add multi-spec planning** — spec maps, shared decisions, grilling, audit, staleness, unified dependencies.
7. **Materialize tasks** — freeze revisions and create board work idempotently with provenance.
8. **Add adaptive depth** after fixed persistence and child planning are reliable.
9. **Complete Workers/Agent policy** — resolve Interact behavior; add capability inheritance/restriction.

Every phase ships a store → core → Tauri → provider → screen vertical path. Fixture/jsdom tests do not replace live-window coverage.

## Acceptance criteria

1. Amendment, feature, and project workspaces resume after app restart and agent replacement.
2. Approved workspaces contain reviewed specs/tasks; spec maps alone cannot be approved.
3. Project workspaces cover all in-scope use cases and validate cross-spec dependencies.
4. Approval freezes one revision and retry creates no duplicate board work.
5. Plan, Manage, Review, Workers, Telemetry, Knowledge own independent scope.
6. Rail matches this plan; global project selector and Interact are absent.
7. Dispatch retains Autorun/Schedules/Runs without rail duplication.
8. Every unique Interact capability is preserved or explicitly retired.
9. Agent capability resolution cannot exceed project policy; runs record effective snapshots.
10. Tauri never falls back to fixture planning/navigation after IPC failure.
11. Rust rejects cross-project identifiers.
12. Existing bounded synthesis, audit, stable-ID, decomposition, and replanning tests remain valid.

## Open decisions

- Workers: one home conversation or multiple durable/disposable sessions.
- Which Interact branch actions Workers retains and where they render.
- Project approval: materialize all tasks or freeze all and release a selected tranche.
- Review: remember last local project or always open chooser.
- Draft deletion: hard delete or tombstone; approved revisions are never hard-deleted.

## Verification

```bash
test -f PLANNING_WORKSPACE_PLAN.md
test "$(wc -l < PLANNING_WORKSPACE_PLAN.md)" -le 300
rg -q '^## Reason for existence$' PLANNING_WORKSPACE_PLAN.md
rg -q '^## Navigation$' PLANNING_WORKSPACE_PLAN.md
rg -q '^## Agent capability policy$' PLANNING_WORKSPACE_PLAN.md
rg -q '^## Migration plan$' PLANNING_WORKSPACE_PLAN.md
rg -q '^## Acceptance criteria$' PLANNING_WORKSPACE_PLAN.md
```

## Next step

Resolve open decisions. Once accepted, update `PLAN.md` and write superseding spec/tasks before production code.
