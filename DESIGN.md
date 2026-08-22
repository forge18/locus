# Locus design

**Status: adopted — v2.** `docs/design_handoff_locus_v2/` is the visual reference. Its HTML,
standalone bundle, and `support.js` are source material for review and fixture extraction; none is
production code.

## Authority

| Source | Owns | Precedence |
| --- | --- | --- |
| `PLAN.md` | architecture and product decisions | highest |
| **This file** | interface purpose, information architecture, and non-negotiable UX rules | design authority |
| `.specs/design-v2/spec.md` | v2 reconciliation and acceptance | implementation contract |
| Feature specs | feature detail and runnable tasks | implementation detail |
| `docs/design_handoff_locus_v2/` | high-fidelity geometry, copy, states, and tokens | visual reference |

The v2 handoff supersedes the removed v1 handoff. It is authoritative for the 31-screen inventory,
the project-scoped rail, dark tokens, and visible interactions. It does not override `PLAN.md`.
Where it changes an architecture decision, the reconciliation in `PLAN.md` wins.

## Product purpose

**Comprehension is the product.** Locus must make an agent's purpose, progress, evidence, and required
human action legible without turning the user into a transcript reader. Density is intentional; poor
alignment, ambiguous status, and unbounded work are not.

Every surface answers one of these questions:

- **Inbox:** what requires my decision, and why now?
- **Dashboard:** what is happening across projects?
- **Project views:** what may run here, with which repo, context, extensions, tools, and routing?
- **Plan:** what was decided, what is the evidence, and what becomes board work?
- **Develop / Automate / Review:** what is running, what changed, and what proof supports it?
- **Dispatch:** what may start, what is queued, and how do I stop it safely?
- **Memory:** what survives a run, what is in the current context, and what is reviewable knowledge?
- **Workshop:** what reusable capability is configured or authored?

## Information architecture

The rail is project-scoped, not a global project filter.

| Scope | Views |
| --- | --- |
| Global | Inbox, Dashboard, Projects, Dispatch, Memory, Settings, Workshop |
| Selected project | Plan, Develop, Automate, Review |

The selected-project card is persistent and visibly names the active project. Global screens state
their scope rather than silently inheriting it. Dispatch has a single status dot: green for live runs,
amber for armed autorun with no run, and red for a fully stopped dispatcher.

Memory expands to Short-term, Long-term, Artifacts, and Wiki. Workshop expands to Agents, CLI,
Commands, Harnesses, Hooks, Linters, Output styles, Providers, Rules, Skills, and Workflows. Workflows
is authoring only; run state appears in Dispatch, Automate, and Review.

## Visual rules

- Dark v2 and a cool-neutral Light theme ship. In both, `--ac` means **human action / focus** and
  `--ac2` means **machine activity**; they never substitute for each other.
- Themes are value sets, not component forks. `theme-system` separates theme values, semantic roles,
  and component aliases under `[data-theme]`; a later theme supplies values and fixtures only.
- Data magnitude uses `--data-1…3` and `--data-hi`; accent is never a chart bar or broad fill.
- Selection is an amber inset ring over `--sf2`. Status remains `--ok` / `--bad`.
- Inter is 400/500 only; JetBrains Mono identifies paths, IDs, models, and numbers. Do not introduce
  600+ text weight.
- The shell is fixed at desktop density. Panes are resizable in the application; the mockup's widths
  are defaults, not a responsive layout contract.
- The only motion is an `--ac2` live pulse and a caret blink. Navigation is instant.
- Keyboard focus is a 2px amber outline with 2px offset. Hover lifts one surface step; pressed lifts
  one further.

## Interaction rules

- The running-agent pill opens a complete active-session popover; the rail dot is the ambient
  dispatch answer, not a second notification channel.
- Project switching type-filters and highlights matches. It changes the selected project only; it
  does not retarget unrelated global screens.
- Provider credentials are represented by OS-keychain references. The host broker alone resolves
  them at the egress boundary; reveal/replace and connection tests never leak a secret to app state,
  logs, artifacts, or a container.
- A plan is editable. Saving a changed requirement re-audits only that requirement. Stage 8 explicitly
  maps tasks to board cards before the final approval.
- Stop all names its affected runs, autorun settings, and schedules. It preserves branches, artifacts,
  and memory; restore is bounded to ten minutes.
- Workflow Visual contains only executable graph structure. Governance owns goal, guardrails, and
  success criteria; each success criterion names its checker.

## Guardrails

1. **No transcript-first review.** Review uses artifacts, diffs, evidence, and walkthroughs.
2. **No ambiguous routing.** A harness needs an adapter and configured provider before selection.
3. **No silent scope shift.** Project scope, disabled extensions, Minisign-verified tool scope,
   router choice, and dispatch state are visible in the UI and stored by the core.
4. **No unbounded dispatch.** Global and per-project caps, queue priority, and optional preemption
   apply before a run starts. Preemption pauses only at an iteration boundary and retains the handoff.
5. **No destructive stop.** Stop all and individual cancellation preserve durable work and explain the
   state transition.

## Superseded v1 ideas

The v1 all-project filter, seven-category rail, strip-first shell, Status view, v1 Workshop tab set,
and workflow canvas carrying run state are superseded. Historical rationale remains in Git; it is not
an implementation contract.

## Verification

`pnpm -C apps/desktop test` must cover rail scope, provider-secret redaction, project/harness gating,
plan card decomposition, dispatcher state transitions, and workflow governance routing. Visual fixture
checks read the v2 screen inventory and token rules from `.specs/design-v2/`.
