# design-desktop

> Superseded by `design-revision` for the current desktop authority; this file records M0.6 history.

**Milestone** M0.6 · **Depends on** M0.5 · **Blocks** M1 runtime settings and every future desktop
surface that consumes their state.

## Purpose

Adopt the reviewed current mockup in `docs/UI mockups for PLAN.md/` without porting its HTML or authoring runtime.
The implementation remains SolidJS/TypeScript in `apps/desktop` and Rust in `locus-core`. This feature
replaces v1 fixture contracts with desktop fixtures and makes the architectural changes explicit before
runtime work depends on them.

## Governed by

- `PLAN.md` §Decisions and §desktop desktop revision
- `DESIGN.md`
- `docs/UI_MOCKUP_REVIEW.md` and the current HTML references under `docs/UI mockups for PLAN.md/`;
  all 31 screenshots

## Contract

### Shell and screen inventory

The desktop shell has a 42px title bar, a 212px rail, a selected-project card, and a running-agent pill
that opens the active-session popover. It replaces v1's all-project filter and category tab bar.
Global views are Inbox, Dashboard, Projects, Dispatch, Memory, Settings, and Workshop. Plan, Develop,
Automate, and Review are selected-project views.

Fixture coverage is the 31 desktop screens named in the handoff README. Screens use the semantic desktop tokens:
`--ac` for human action and focus, `--ac2` for machine activity, `--data-*` for magnitude, and
`--ok`/`--bad` for outcome. The dark UI does not use accent as a chart bar or broad fill. The concrete
theme layering, root preference, and theme acceptance live in `theme-system`; M0.6 ships both desktop Dark
and a cool-neutral Light theme.

### Provider, harness, and tool policy

A provider has an identifier, authentication method, OS-keychain reference, optional base URL,
verification metadata, and curated model catalog. The database stores only the keychain reference.
Model aliases are provider-owned and selectors display aliases rather than raw IDs where present.

A harness is selectable only if an adapter exists and the project permits it. It declares compatible
providers and defaults for model and effort. Autorouting is settings policy: six complexity bands
(`xtra-low` through `max`) each choose model, effort, approval requirement, and prose. Missing bands
fall upward.

CLI tools are installed eagerly into image layers from the enabled catalog. Project and role scopes
only remove tools from that baseline. User tools require a Minisign-signed manifest and binary; Locus
settings hold trusted public keys, and unsigned or untrusted uploads are rejected before entering an
image.

### Project, plan, and dispatch policy

Project Settings owns harness allow-list/default, repos, base context, extension enablement, and CLI
role scope. Analytics consumes stored run/model/token/cache/spend data.

Planning has nine stages. `Decompose` maps the approved spec into board cards as spec-only, every task,
or spec plus selected carve-outs. The mapping is editable and becomes durable only with final approval.

Dispatch owns autorun, schedules, queue ordering, global and per-project parallel caps, optional
iteration-boundary preemption, and Stop all. Stop all is reversible for ten minutes and preserves
branches, artifacts, and memory.

### Workflow authoring

Workflows have a versioned graph plus Governance: goal, named guardrails with prompt bodies, and
success criteria. A criterion has a kind (`command`, `assertion`, or `human`) and a named checker.
The authoring surface contains no execution state.

## Supersedes

| Existing feature | desktop replacement |
| --- | --- |
| `app-shell`, `navigation` | project-scoped rail and running-agent popover |
| `design-system`, `fixtures`, all `screens-*` | desktop tokens, theme layering, Dark/Light fixture set, and 31-screen fixture set |
| `harness-registry`, `agent-definitions`, `sandbox`, `materializers` | provider aliases, adapter gate, policy routing, selected extensions/tools |
| `planning-module`, `board` | Decompose and durable spec/task-to-card mapping |
| `guardrails`, `workflow-engine`, `schedules` | queue policy, safe stop-all, workflow Governance |
| `memory`, `tool-compaction`, `dashboard-metrics` | desktop Memory and Project Analytics viewers |
| `marketplace-index`, `marketplace-installer` | Minisign-trusted CLI catalog and image installation contract |

## Acceptance

1. Every desktop screen has a named fixture route; v1-only screens are unreachable.
2. No fixture, log, Tauri event, or persisted row contains a provider secret.
3. A project cannot select an adapter-less harness or a provider it has not configured.
4. Router fallback is upward and recorded with the selected band, model, effort, and approval state.
5. Disabled extensions and unavailable tools do not enter the materialized config or image.
6. Plan decomposition produces the exact approved card set and preserves task dependencies.
7. Queue caps and preemption are enforced before start; preemption occurs only at an iteration boundary.
8. Stop all records its scope, stops runs/autorun/schedules, and preserves branches, artifacts, memory,
   and the ten-minute restore snapshot.
9. Workflow execution state cannot be rendered by the Workflow authoring routes.

## Open

- Keychain implementation must support macOS, Windows, and Linux without changing the stored
  credential-reference contract.
