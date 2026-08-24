# setup-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision` · **Blocks** `workshop-revision`, `dispatch-revision`

## Purpose

Give the Setup rail category — the project-scoped screen at `locus://projects` — its full contract:
three tabs, **Settings**, **Persistence**, **Analytics**, each specified down to footer copy and
empty states. Settings is five panels of project-local policy (harnesses, repos, extensions, CLI
tools, base context). Persistence is new: a grouped read of everything the project has kept —
memory, specs and tasks, research — with deletion offered only where PLAN.md's own memory model
allows it. Analytics reuses the global Analytics shape, scoped to one project.

## Governed by

- `PLAN.md` §Adding a repo, §Materializers — the code half of the contract, §Memory,
  §Token discipline, §M5 — Project management
- `docs/UI_MOCKUP_REVIEW.md` — Setup (project)

## Contract

### Settings

Five panels, each with its own persistence root and footer copy; none of the five is optional.

- **Harnesses.** Table `Harness · Adapter · Provider · model · Agent default · (remove)`. A harness
  with no working adapter is listed but not selectable — the allow-list can name it, the router
  cannot route to it. Exactly one row carries the agent default, and it must already be in the
  allow-list. The allow-list is an **ordered** list, not a set: "enabled harnesses are offered to the
  router in the order listed; anything the router does not claim runs on the agent default." Add
  harness draws only from harnesses configured in Workshop and not yet added here ("Configured in
  Workshop, not yet added here."); the empty state for an already-complete table is "Every harness
  with a working adapter is already here."; the empty state for the allow-list itself is "No
  harnesses here yet — unattended agents in this project have nothing to run on." The routing logic
  named in the footer is authored once per harness, under Workshop → Harnesses — this panel only
  toggles membership and order.
- **Repos.** A repo belongs to exactly one project. Reassigning a repo re-tags every run, artifact,
  and memory fact that came from it to the new project; the old project tag is kept on the historical
  record rather than overwritten, so history does not silently change project. Each row carries
  origin, branch summary, activity, and a project chip with a caret that performs the reassignment.
- **Extensions.** Seven toggle groups — agents, commands, hooks, linters, rules, skills, styles —
  pulled from Workshop's defaults. Each group carries a tri-state master (on / off / mixed) plus a
  per-item toggle. Output styles additionally carry a "set default" chip; the extension currently
  the active default cannot be switched off until another style is chosen to replace it. Switching an
  extension or an entry off removes it from the materialized tree on the next run — it does not
  delete the authored definition, matching `ProjectExtensionScope`'s subtraction-only contract in
  `crates/locus-core/src/harness/materialize/extensions.rs`.
- **CLI tools.** Left: catalog search with a match count and per-row Add/Remove. Right: "In this
  project · n" with per-row removal, backed by `ProjectToolScope` in
  `crates/locus-core/src/services/tools.rs`. Adding or removing a tool rebuilds the project's
  container image once; it does not rebuild per run, and a tool an agent cannot find is named as the
  second most common reason a run stalls.
- **Base context.** Exactly one `base.md`, and there is no second. A token budget meter, a card
  showing version, edit time, and run count, plus History and Save. Over budget reads as "something
  belongs in a skill or a rule instead" rather than as an error — the budget is a design signal, not
  a hard limit.

### Persistence (new)

Intro copy: "Everything this project has kept, in one place. Memory tiers decay on their own
schedule; specs and research stay until you delete them."

Three groups, each a page of sections at four items with a "Show all n" / "Show fewer" toggle, and
items that expand to their body:

| Group | Sections | Delete |
| --- | --- | --- |
| Memory | Short-Term ("clears at session end unless promoted"), Long-Term ("promoted facts — survive the session"), Artifacts ("what runs left behind") | Long-Term and Artifacts only |
| Specs & Tasks | one section; items carry the plan body and a nested task list whose rows navigate to the board | none |
| Research | one section; source and synthesis items | none |

Short-Term, Specs & Tasks, and Research carry no delete control at all — Short-Term because it is
already governed by its own decay (PLAN.md §Memory), Specs & Tasks and Research because they are the
written record PLAN.md §Memory calls the layer that "dies when" never, owned by the repo. Long-Term
and Artifacts are the two sections where a user action, not decay or git, is the only way a record
goes away.

### Analytics (per project)

Same shape as global Analytics, scoped to the project. Specified in full by `analytics-revision`;
this spec does not restate it and adds no per-project deviation beyond scoping.

### Header

Project name, locator, a three-way segmented control **Settings / Persistence / Analytics**, then
**Archive** and **Rename**.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `desktop-project-operations` — project-configuration half (tasks 1–14, 41–45: settings root, harness allow-list/default, repos, base context, extension overrides, tool scope, analytics aggregation, project list/lifecycle, Extensions/CLI rendering) | this spec |
| `desktop-project-operations` — planning half (tasks 15–26, 46–47: nine-stage planning, spec editor, task decomposition, card modes) | `plan-revision` |
| `desktop-project-operations` — dispatch half (tasks 27–40: parallel caps, priority policy, autorun, schedules, queue/runs, Stop all) | `dispatch-revision` |

`desktop-project-operations` tasks 48–52 (Automate/board rendering) belong to neither this spec nor
`plan-revision`/`dispatch-revision`; the "Automate" rail category itself is retired by
`design-revision`, and its replacement is out of scope here.

## Acceptance

1. The Harnesses table renders every listed column, lists an adapter-less harness without letting it
   be selected, and enforces exactly one agent default that is itself allow-listed.
2. The harness allow-list preserves the order it was given, and that order — not alphabetical or
   insertion-into-a-set order — is what the router-precedence footer describes.
3. The Add-harness picker offers only harnesses configured in Workshop and not already on the
   allow-list, with both named empty states reachable.
4. Reassigning a repo updates the project tag on new activity while the prior project tag remains
   readable on every run, artifact, and memory fact recorded before the move.
5. A project cannot list the same repo under two projects at once.
6. Disabling an extension group, an individual extension, or an entry removes it from the next
   materialized tree without deleting its authored definition.
7. The active default output style cannot be disabled while it remains the default; disabling
   succeeds only after another style is set as default.
8. A CLI tool add or remove triggers exactly one image rebuild for the project, not one per run.
9. Exactly one `base.md` exists per project; its token budget meter reflects the stored budget, and
   the over-budget state reads as a design signal, not an error.
10. The Persistence tab renders three groups — Memory (three sections), Specs & Tasks (one section),
    Research (one section) — each paged at four items with a working "Show all" / "Show fewer".
11. Delete is reachable only from Long-Term and Artifacts items; no delete control renders on
    Short-Term, Specs & Tasks, or Research.
12. A task row inside a Specs & Tasks item navigates to that task's board card.
13. The Analytics tab renders the project-scoped shape defined by `analytics-revision` and adds no
    contract of its own.
14. No file under `.specs/` cites `desktop-project-operations` as governing the project-configuration
    contract after this spec lands.

## Open

- Whether `RoleToolScope` (per-workflow-role tool subtraction, distinct from `ProjectToolScope`)
  surfaces anywhere in Setup or stays exclusive to the workflow/dispatch surfaces — not decided here;
  the CLI tools panel in this spec is project-scoped only.
- `services/memory.rs`, `services/wiki.rs`, and `services/board.rs` are currently stubs. The section
  and delete-scoping contract above is fixed regardless of how those services land; the module paths
  in `tasks.md` name where that implementation is expected to go.
- Whether Research items carry a live link back to their wiki source page, or stand alone as a
  Persistence-only copy, is undecided pending the wiki service's implementation.
