# navigation

> **Historical M0.5 contract.** Its seven-category rail and all-project filter are superseded for
> new work by `.specs/design-desktop/spec.md` §Shell and screen inventory.

**Milestone** M0.5 · **Depends on** `app-shell` · **Blocks** every `screens-*`

## Purpose

The view/category/locator map, and the resolver behind it. PLAN.md is emphatic that the locator scheme
lands with the **first** navigation rather than being retrofitted: the palette, global search, inbox
items, board-card links, artifact comments, deep links, and a detached window's identity are otherwise
seven navigation paths that drift apart, and against one locator they are one resolver with seven
callers.

## Governed by

- PLAN.md §One address space, so there is one resolver — the locator grammar
- PLAN.md §Navigation — seven categories; operate versus author; project as a filter
- PLAN.md §Three rules that keep it from sprawling
- `docs/UI_MOCKUP_REVIEW.md` §Navigation
- `.specs/design-desktop/spec.md`

## Contract

**Views → category, label, locator:**

| View | Category | Rail label |
| --- | --- | --- |
| `inbox`, `status` | `dashboard` | Inbox |
| `plan` | `plan` | Plan |
| `develop` | `develop` | Develop |
| `board`, `tasks` | `automate` | Automate |
| `telemetry`, `runs`, `artifact` | `review` | Review |
| `extensions`, `agents`, `canvas`, `plugins` | `workshop` | Workshop |
| `wiki` | `wiki` | Wiki |

**Tabs per category** — Dashboard `[Inbox, Status]`; Automate `[Kanban, List]` **in that order**;
Review `[Telemetry, Runs, Artifacts]`; Workshop `[Extensions, Workflow, Plugins]`. Plugins groups
CLI Tool, Harness, and Provider; Extensions groups the eight extension types and Workflows. Plan,
Develop and Wiki have none.

**Rail click lands on the category's first view:** Inbox→`inbox`, Automate→`board`, Review→`telemetry`,
Workshop→`extensions`.

**`agents` is a top-level Workshop view, not a drill-down.** The Workshop rail item lands on it and
the rail lights **Workshop** while it is open. Automate has no Agents tab: task detail exposes a
task's root session and child-agent run tree.

**Locator grammar**, the one address space:

```
locus://<project>/session/<id>[/run/<id>]
locus://<project>/task/<id>       artifact/<id>       page/<slug>
locus://<project>/workflow/<id>[/execution/<id>]      agent/<name>@<version>
```

**One resolver, seven callers.** `resolve(locator) → { view, params }` and `locate(view, params) →
locator` are inverses. `Cmd-K` resolves a locator; `Cmd-P` searches for one; back/forward per window is
a stack of them.

**Three rules, enforced not documented:**

- **Detail opens in place** — as a sheet over the current category, never a new category or window.
- **One viewer per kind, several entry points** — an artifact renders identically however you reached it.
- **The category list is closed** at ten; a new surface joins one rather than adding an eleventh.

**Project is a scope filter, defaulting to all.** You filter, never switch — switching means leaving
somewhere that was still running.

## Acceptance

1. Every view in the table maps to its category, and the rail lights that category.
2. Each category shows exactly its tab set; Plan, Develop and Wiki show none.
3. Rail clicks land on the documented first view, not on whatever was last open in that category.
4. Opening `agents` keeps **Workshop** lit — it is a Workshop view, not a drill-down. No Agents tab
   exists in the Workshop bar.
5. `resolve(locate(v, p))` equals `(v, p)` for every view — round-trip asserted, not assumed.
6. Every navigation entry point calls the resolver. A component that sets `view` directly is a failure.
7. Opening a detail renders a sheet over the current category; the rail does not change.
8. Changing the project filter never changes the view.

## Open

- Whether back/forward is per window or global. PLAN.md says per window; nothing at M0.5 depends on it,
  and the second window does not exist until M1.
