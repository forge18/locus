# shell-revision

> **Navigation revision:** `planning-workspace` supersedes this spec's global project switcher, rail groups, and category map. The title-bar pill and locator mechanics remain historical shell constraints until their replacement tasks land.

**Milestone** M0.7 · **Depends on** `design-revision` · **Blocks** every M0.7 screen feature

## Purpose

Build the shell `design-revision` settled the vocabulary for: the 42px title bar and its two pills,
the 212px rail's Project/Cross-Project split, the category→landing-view map, the `locus://` locator
scheme and its ⌘K palette, the toast stack, the shared Merge modal, and the Inbox screen the Dispatch
and Inbox pills open into. Every other M0.7 screen feature renders inside this shell and is reached
through it — nothing else is a route it cannot resolve.

The screen-by-screen contract lives in
[`docs/UI_MOCKUP_REVIEW.md`](../../docs/UI_MOCKUP_REVIEW.md), sections **Navigation** and **Inbox**;
this spec states the contract and its invariants and does not restate pixel geometry.

## Governed by

- `PLAN.md` §Navigation (`Navigation — seven categories, one address space`), §The user inbox,
  §Decisions already made (`Navigation` row)
- `docs/UI_MOCKUP_REVIEW.md` — Navigation and Inbox sections
- `.specs/design-revision/spec.md` — the retired-category invariant, the 29-view inventory, the
  `locus://<project|all|app>/view/<route>` locator scheme, and the seven-stage plan pipeline this
  shell's Plan category link points at

## Contract

### Title bar

42px. Native traffic lights, the `LOCUS` wordmark, the current category and view label, then the
Dispatch pill and the Inbox pill, in that order.

### Dispatch pill

Broadcast icon, running count, a pulsing dot while any run is active. Opens an activity popover with
two tabs, **Attention needed** and **All**. Rows carry icon, title, elapsed, project tag, meta.
Footer: **Stop all** (destructive) and **Open Dispatch** (lands on Dispatch → Autorun, its first tab).
Tab copy is load-bearing and differs by tab:

- Attention needed: "Three runs are blocked on you. Everything else is running and does not need a
  decision."
- All: "Runs first, then what has already happened. Nothing here is an obligation unless it is in the
  first group."

### Inbox pill

Tray icon, count badge. Opens a quick-preview popover listing items that need a response. Footer:
**Open Inbox**, which lands on the `inbox` view.

### Rail

212px, two groups.

- **Project** — a switcher pill (`#project`) with type-to-filter and match highlighting, a
  per-project running/spend note, and a **+ New project** row; then the project-scoped categories
  **Setup, Plan, Manage, Interact, Review**.
- **Cross-Project** — **Analytics, Memory, Settings, Workshop**. Memory expands to **Short-term,
  Long-term, Artifacts, Wiki**. Workshop expands into **Plugins** (CLI Tool, Harness, Provider) and
  **Extensions** (Agents, Commands, Hooks, Linters, Output styles, Rules, Skills, Workflows).

Retired project, task, metrics, and project-list labels never reappear as a rail label, route id,
or fixture name in this or any feature this spec governs.

### Category → landing view

A category click lands on that category's first view. The map:

| Category | Landing view |
| --- | --- |
| Setup | `projects` |
| Plan | `plan` |
| Manage | `sessions` |
| Interact | `interact` |
| Review | `qa` |
| Analytics | `status` |
| Memory | `short` (its first expansion item, Short-term) |
| Settings | `settings` |
| Workshop | `agents` (its first expansion item, Agents) |

Dispatch and Inbox are not rail categories — they are title-bar pills. Their views (`autorun`,
`schedule`, `runs`, `inbox`) are reached only through a pill, the ⌘K palette, or a locator, never
through the rail.

### Locators

Every view in scope carries a `locus://<project|all|app>/view/<route>` locator. The rail, the two
pills, the palette, and every in-shell link resolve through one shared resolver; no component may
navigate by constructing a route string itself.

### ⌘K locator palette

Sections, in order: **Needs you**, **Running now**, **Where you were**. Each row carries its locator.
Footer: "Opens on a list — recognition, not recall." Key hints: `↑↓ move · ↵ open · ⇧↵ scope ·
esc close`.

### Toast stack

Bottom right, dismissible. Suppressed on Interact and while the Dispatch popover is open — those are
the two surfaces where a transient toast would either compete with the agent panel or get lost behind
an open popover. Footer: "Nothing here needs an answer. Things that do go to your inbox."

### Merge modal

Shared overlay, opened from any surface that lands a branch. Header: "The work becomes yours." Names
the branch, the commit split, and "the only irreversible step in the run." Two columns:

- **Evidence travelling with it** — verify command and exit code, plan clauses satisfied, analyzer
  result, and how many changed files were actually opened.
- **Size of the change** — files, added, removed, and whether the change is inside the guardrail.

A warning box names files that were not opened: "an approval granted without opening the artifact is
the measured failure mode, not a hypothetical one." Buttons: **Merge and close the task**, **Open the
two files first**. The merge is recorded as a review, not a steer.

### Inbox screen

Two panes.

**Left** — **To do** / **Completed** tabs; a throughput strip (`3 / 6 per hour`, "under budget"); a
per-view project filter ("Filters this list only. Every other screen keeps its own."). The list is an
`aria-live` log.

Three item types — **Gate**, **locus ask**, **Guardrail** — each documenting the response it wants.
Footer: "Every item type documents the response it wants. Items without a response belong in Activity."

**Completed** groups resolved items by day with time-to-resolve: "Kept so the resolution is
auditable — what you decided, and how long a loop waited on you for it."

**Right** — the Gate detail pane: tag, title, locator, agent and role, gate mode (`human`); the plan
under review; an info callout naming the irreversible step; a comment box ("Comment steers the agent
that made it"); then **Approve & release the loop** / **Send back with comment**. Two footnotes:

- *Why this is here* — which workflow node, set to `human`, and that the agent is blocked, not idle.
- *Cost of waiting* — "One loop held for 4m. No tokens burn while blocked."

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `desktop-application-shell` | this spec, where the two differ; `desktop-application-shell` remains the record of what M0.6 built |

## Acceptance

1. The title bar renders at 42px with traffic lights, wordmark, category/view label, Dispatch pill,
   and Inbox pill, in that order.
2. The Dispatch pill's popover renders exactly two tabs, Attention needed and All, and each renders
   its footer copy verbatim as stated in this spec.
3. The Dispatch popover footer renders Stop all and Open Dispatch; Open Dispatch resolves to
   `autorun`.
4. The Inbox pill's popover footer renders Open Inbox, which resolves to `inbox`.
5. The rail renders exactly two groups — Project and Cross-Project — with the categories and
   expander contents listed in this spec, and no others.
6. No rendered rail item, route id, or fixture under this feature's scope uses a retired category
   label or a view-level `Projects`.
7. Every rail category resolves to the landing view named in the category→landing-view table on
   first click.
8. Dispatch and Inbox views are unreachable from the rail and reachable only via their pill, the ⌘K
   palette, or a locator.
9. Every view named in this spec resolves a `locus://<project|all|app>/<kind>/<id>` locator through
   one shared resolver.
10. The ⌘K palette renders its three sections in order, each row carrying a locator, and the stated
    footer and key hints.
11. The toast stack renders on every view except Interact, and renders nothing while the Dispatch
    popover is open.
12. The Merge modal renders both columns, the warning box, and both buttons; it never auto-merges.
13. The Inbox screen renders To do and Completed tabs, the throughput strip, and the per-view project
    filter; the item list carries the `aria-live` role.
14. A Gate item's detail pane renders both response actions and both footnotes; selecting Send back
    with comment requires non-empty comment text.
15. Completed items render grouped by day with a time-to-resolve value on each.
