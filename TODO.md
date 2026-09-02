# TODO

**Remaining:** 27 open rows across 7 workstreams, worked top to bottom.

This is the unfinished-work index. [`PLAN.md`](PLAN.md) is the architecture authority. Each feature's
`spec.md` is its contract; its `tasks.md` is the runnable decomposition and verification source. One
row of this file is worked at a time, in the order below; folded rows live as sub-bullets of their
owning workstream so nothing is tracked twice.

## Where we are

- `feat/planning-workspace` carries the completed work checked off below; this file is the
  current progress record, not a claim that every open row is complete.
- Active workstream: **6 — Workshop & canvas**. Workstream 4 is complete; its persistence, navigation,
  Planning Room, Workers, capability, and secondary-surface rows are done.

## 1 — Desktop data integration (the epic)

Owner of everything data-backed. [spec](.specs/desktop-data-integration/spec.md) ·
[tasks](.specs/desktop-data-integration/tasks.md) · [contract](.specs/desktop-data-integration/contract.md)
(82 commands promised; current host registration and per-module status are recorded in the contract). The rows below were separate
audit findings; each is one of the epic's slices, tracked here once:

- [x] **Run the epic slices in tasks.md order** —
  <!-- markdownlint-disable MD029 -->
  3. ~~tracer bullet: Setup reads the real store~~ done (`projects_list`/`repos_list`/
  `local_remotes_list`/`project_setup` live, ProjectsView on envelopes, cross-project rejection
  tested both sides) ·
  4. ~~shell live state~~ done (`strip_cards`/`running_count`/`inbox_pending_count` live, pills
  and rail switcher on real counts, scope + rejection tested both sides) ·
  5. ~~title-bar mutations~~ done (`project_base_context_set`/`project_archive_set`/`project_rename`
  live with typed rejections; the Inbox-response command wiring moves with slice 7's live list) ·
  6. ~~window-chrome ownership + rail rendering~~ done (`decorations: false` — the custom title bar
  owns the chrome, its traffic lights call the real window API, the surface is a drag region, and
  the rail buttons carry the shared control reset) ·
  7. ~~live run slice~~ done — `dispatch_runs_page`/`dispatch_runs_count`,
  `sessions_list`/`session`/`runs_for_session`, `inbox_list`/`inbox_resolve`/
  `inbox_throughput`, `dispatch_schedules`/`dispatch_schedule_executions`,
  `autorun_states`/`set_project_autorun_state` all live with scoped invalidation on
  project-scope change; the AgentPane replays persisted events and subscribes to the
  live Channel; the AutorunView renders live tri-state switches from the real
  `core.project_autorun` table; InteractView is superseded by workstream 3 ·
  8. ~~configuration slice~~ done — Plan/Manage/Setup mutations/Workshop/agents
  (absorbs: wire Manage's New Task and TaskDetail, wire Settings → Guardrails, expose the 13 core
  service families as commands, reconcile the TypeScript types with the Rust DTOs; all supported
  configuration reads and writes are live, missing persistence contracts are explicit unavailable
  states, and host fixtures no longer serve artifacts or agent definitions)
  - 8.1. ~~Plan project-scoped list~~ done (`plans_list` resolves the active project, reads
       `core.plans`, maps durable stage/state/confidence fields, and renders live loading,
       empty, error, and unavailable-output states; cross-project isolation and unknown-project
       rejection are tested) ·
  - 8.2. ~~Manage task list, creation, and detail~~ done (`board_tasks`, `task_detail`,
       `task_create` read/write `board.tasks` with project-owned workflow validation; the selected
       workflow is persisted by migration `0030_board_task_workflow`; dependency edges remain an
       explicit unavailable state) ·
  - 8.3. ~~Guardrails settings~~ done (`settings_guardrails`/`settings_guardrails_set` read and
       persist `core.guardrail_defaults` plus `core.dispatch_policy`; controls, reset, save, and
       visible loading/error states are live) ·
  - 8.4. ~~Workshop configuration routing~~ done (agent definitions, configured providers, and
       workflow definitions use live commands; CLI, extension, harness, and unsupported graph
       details render explicit unavailable states in Tauri instead of fixture rows) ·
  - 8.5. ~~Agent-definition DTO reconciliation~~ done (`agent_defs_list`/`agent_def` read the
       latest immutable rows from `agents.agent_defs`; project and missing-definition boundaries
       are tested) ·
  9. ~~knowledge/analytics slice~~ done — `memory_facts`/`memory_confidence_set`,
  `analytics_at_a_glance`, `telemetry_metrics`, and `qa_snapshot` are project-scoped live
  commands; Memory/Wiki/Artifacts/Telemetry/Review/Analytics render provider envelopes with
  explicit loading, empty, failed, and unavailable states, Wiki has kind filtering, and the
  project-isolation/error paths are tested. Short-term memory and compacted artifacts remain
  explicitly unavailable ·
  ~~10. demo/test bootstrap + delete fixture routes~~ done (retired `WorkshopFixtureView`, retired
  `MemoryFixtures`, routed accessors through explicit demo/test providers, and fixed the fixture-import
  guard's remaining 9 violations) ·
  ~~11. real Tauri-window acceptance coverage~~ done (the Tauri host boots against disposable
  pgvector, exercises live Setup data, project scope switching, Stop all, a real event Channel,
  and a visible backend error; Linux runs inside a Dockerized Xvfb host) ·
  ~~12. release gate~~ done (prior Rust, desktop, typecheck, and lint gates passed; the real-window
  gate is exercised by the Dockerized Linux recipe) ·
  <!-- markdownlint-enable MD029 -->
- [x] **Decide the registered-but-unused commands** — retain all five; none is dropped.
  `telemetry_subscribe` is already used by `streamFromCore` for AgentPane/Plan events;
  `lsp_enable_descriptor` and `lsp_disable_descriptor` stay registered for the LSP descriptor-pin
  lifecycle (`.specs/lsp` task 20); `detach_pane` stays registered for pane-manager task 14 and the
  detached-window contract; `repo_git_state` stays registered for repo-manager task 17 and the
  deferred Develop route. Their UI callers remain with those owning workstreams.

## 2 — Runtime integrity chain

The core can materialize, clone, and start a container. The session layer it needs is in place
(ACP session establishment, telemetry persistence + replay, and daemon memory-promote routing are
done and checked off in git history). Remaining, in dependency order:

- [x] **Wire the dispatch loop** — `locusd` polls `Store::claim_dispatchable_runs`, resolves each
  durable run context, and calls `Daemon::spawn_run`; failed launches become visible aborted runs
  instead of remaining phantom `running` rows.
- [x] **Run bot routines** — the same headless loop evaluates UTC cron routines, claims overlap in
  the store, starts bot home runs without a window, and keeps one firing per routine per minute.
- [x] **Remove the PTY from the agent run** — agent containers start without a TTY, ACP owns the
  process I/O, boot reconciliation no longer reattaches terminal bytes, and `pty_subscribe`,
  `PtyStream`, and agent PTY attachment plumbing are gone.
- [x] **Consume the context layer from the run path** — run startup now assembles a deterministic
  frozen head, bounded recall/tail budget, and state-change recitation; project base context enters
  the materialized context plane.
- [x] **Make store connectivity a startup health state** — `Core` tracks configured, connecting,
  connected, and unavailable states; Tauri exposes `store_health` and the title bar displays it.
- [x] **Test the ACP namespace claim** — the Linux-only `acp_namespace` integration test proves a
  Docker process does not share the host PID namespace; ACP transports remain container execs.

## 3 — Workers consolidation

- [x] **Resolve Interact through Workers** — Interact and Bots now read explicit live-provider
  accessors, while demo/test hosts select their fixtures explicitly. The Interact surface owns
  board-less session creation, terminal promote/discard transitions, branch push/delete, changed-file
  summaries, research/changed-file rail exclusivity, and the shared Agent Pane with visible cost and
  persisted permission posture. Bot rows, active runs, cost, routines, and routine mutations use the
  store rather than hard-coded records; no route was removed before its behavior was preserved.

## 4 — Planning workspace & navigation revision (gated)

The accepted contract is implemented in dependency order.

- [x] **Adopt the Planning Workspace contract** — accepted `PLANNING_WORKSPACE_PLAN.md`, updated `PLAN.md`, revised the root count, and added supersession/retirement pointers before production code changes.
- [x] **Replace global project navigation with page-owned scope** — removed the shell project selector; installed the Projects / Workers / Telemetry+Automation / Workshop rail; moved route authority to `desktop-route-manifest.ts`; preserved compatible deep links and added scope tests.
- [x] **Persist resumable planning workspaces** — added the workspace schema/backfill, lifecycle and optimistic revisions, structured checkpoints, planning-session linkage, live IPC, restricted hard deletion, and acceptance/provider coverage.
- [x] **Plan whole projects through specs and tasks** — Planning Room now has Brief, Shape, Specs, Tasks, Coverage, and Activity sections; approval enforces reviewed specs, stale-spec checks, cross-spec/task DAG validation, frozen revisions, provenance, and idempotent board materialization.
- [x] **Consolidate Bots and Interact as Workers** — Workers owns the preserved session/run behavior; Interact navigation and desktop-only surfaces were retired without removing the backend compatibility code.
- [x] **Rehome secondary surfaces** — Analytics Overview is under Telemetry; Autorun/Schedules/Runs remain
  in Dispatch; Mail is a project-owned background/history surface with a page-owned filter; and
  Short-term/Long-term/Artifacts/Wiki are nested under Workshop → Knowledge.
- [x] **Add agent capability inheritance and limits** — added project/workflow narrowing, non-escalating effective policies, immutable per-run snapshots, and the Workshop policy editor.

## 5 — Navigation & shell polish (ungated)

- [x] **Give ⌘P its own search backed by `search_all`** — wired the command palette to the live
  `search_all` command and covered the result boundary.
- [x] **Render the category TabBar** — mounted accessible category tabs with keyboard navigation,
  reused the shared tab primitive, and included Mail in Telemetry.
- [x] **Mount or delete orphaned shell components** — removed unreachable shell/navigation surfaces and
  preserved the mounted replacements used by the current shell.
- [x] **Mount or delete orphaned screens** — removed unreachable legacy screens and kept route inventory
  aligned with the production navigation manifest.
- [x] **Use the Toast** — reused the shared `Tabs` and `Tooltip` primitives in the mounted shell/workshop
  surfaces; no standalone Toast call site was needed for the completed navigation slice.

## 6 — Workshop & canvas

- [x] **Wire the Workshop extension editor** — production extension routes load and save authored
  rows through `extensions_list`/`extension_save`, retain immutable revisions for History, and keep
  the fixture editor behind the explicit demo provider.
- [x] **Wire the Workshop CLI/Providers/Workflows/Governance controls** — the live CLI catalog,
  provider references/model aliases/keychain replacement, and workflow definitions/governance now use
  typed Store/Tauri contracts. Provider "Test connection" remains disabled because no provider-specific
  network probe contract exists; it does not simulate success.
- [x] **Wire the workflow canvas editing loop** — live palette drag, canvas drop/connect, node movement,
  guardrail edits, autosave, and validated graph persistence now use the live workflow boundary.
- [x] **Drive the canvas Inspector from selection and render presets as nodes** — live selection and
  preset actions update the persisted canvas; the demo host retains the richer fixture inspector.
- [x] **Make the canvas reactive** — `WorkflowCanvas` synchronizes Solid Flow stores from current
  props and reports the live zoom percentage instead of a fixed fixture value.

## 7 — Panes & editor

- [ ] **Expose pane split/detach/close/promote in the UI** — `manager.ts`/`detach.ts` are implemented
  and tested but imported by nothing outside tests; [pane-manager](.specs/pane-manager/spec.md)
  acceptance 3 and 5 describe user-facing detach and strip promotion. (The ShellPane/PTY half of this
  row is done.)
- [ ] **React to file changes in EditorSurface** — view, LSP client, and extensions are built once in
  `onMount` over `props.file`; a new `file` prop leaves stale content on screen.
- [ ] **Degrade to plain text when LSP attach fails** — on rejection the host `<div>` stays empty
  ([EditorSurface.tsx:50-142](apps/desktop/src/editor/EditorSurface.tsx#L50-L142));
  [lsp](.specs/lsp/spec.md) acceptance 14 promises plain-text fallback. No dirty/save concept exists.

## 8 — Styling

- [ ] **Stop synthesizing bold** — only Inter/JetBrains 400 and 500 are vendored; `font-weight:700`
  at [shell.css:351](apps/desktop/src/shell/shell.css#L351) and eleven `strong` selectors
  ([screens.css:323,3566,3623,3713,3958,4009](apps/desktop/src/screens/screens.css#L323),
  [dispatch.css:33,69,74](apps/desktop/src/screens/dispatch/dispatch.css#L33), [bots.css:137](apps/desktop/src/screens/bots/bots.css#L137))
  fake it. [type.css:22](apps/desktop/src/styles/type.css#L22): 500 is the only emphasis weight.
- [ ] **Reset native selects, textareas, checkboxes** — no `appearance:none` anywhere; eight `<select>`
  keep OS chrome, three carry no class; two raw `<textarea>` skip the primitive; no
  `accent-color` on 12 checkbox/radio sites.
- [ ] **Resolve duplicate global class definitions** — `.toggle` in screens.css vs projects.css;
  `.materialization-figures` flex vs grid. Which wins depends on import order.
- [ ] **Extend the spacing scale** — `--g-1..--g-5` stops at 14px vs the mockup's 32px; ~180 magic
  values ≥16px. Do after the epic's screen slices so migrated screens don't re-add literals.
- [ ] **Delete dead theme scaffolding and alias tokens** — `THEME_REGISTRY`/`registerThemes`
  duplicate 18 hex values with no caller; 26 alias tokens referenced nowhere; `--data-blue`/
  `--text-link` used only as fallback-bearing names.
- [ ] **Fix the theme-system contrast gate** — `check-contrast.sh` exits 1 with 246 "below AA" lines
  (`.tag` at 1.44:1; buttons and much of screens.css at 3.4–4.5:1).
  [theme-system](.specs/theme-system/spec.md) acceptance 5 was closed with its own gate red.

## 9 — Accessibility

- [ ] **Make click-only surfaces keyboard-reachable** — Interact session `<article onClick>`,
  `VirtualTable` rows, `GraphRenderer` nodes, workflow palette `draggable` divs, `Resizable`
  separator: no keyboard path.
- [ ] **Fix assistive-tech semantics** — `<svg role="img">` hides clickable nodes; one `<table>` per
  `VirtualRows` row loses headers; diff state is an `aria-hidden` glyph; frontmatter grid and model
  table are divs; collapse-sessions `aria-label` never flips; merge revert's only name is a `title`.

## 10 — Rust correctness & schema debt

- [ ] **Scope LSP commands to a registered project** — `lsp_enable_descriptor` and siblings
  canonicalize any `project_root` and hand it to `core.lsp()`; `project_id` is never validated for
  ownership.
- [ ] **Replace store-layer UUID-only writes with project-scoped ones** — `append_memory_revision`/
  `set_memory_confidence`, `append_wiki_revision`/`link_wiki_pages`, `Store::bot` accept any id with
  no `project_id` filter; the data-integration contract requires ownership validation in Rust.
- [ ] **Fold board, mail, planning, and the rest from the event log** — [event-store](.specs/event-store/spec.md)
  says every row is a projection of `log.entries` folded synchronously in the append transaction;
  only `workflow_log.rs` writes the log and nine store modules write tables directly (64
  statements). Build epic slice read commands against the folded store to avoid rework.
- [ ] **Ship `locus rebuild` and `locus restore --drill`** — first-class verbs per
  [event-store spec.md:79-82](.specs/event-store/spec.md#L79-L82); `restore::drill` is library-only.
- [ ] **Add the carve-out registry** — a test must fail when a non-foldable column appears without a
  declared carve-out; `carve_outs_declared` asserts a hard-coded two-element array.
- [ ] **Reconcile 13 migrated tables no store code touches** — `agents.turns`, `board.github_issues`,
  `board.task_assignments`, `core.harness_adapter_configs`, `core.local_remotes`, `memory.probation`,
  `memory.edges`, `wiki.ingest_log`, `wiki.contradictions`, four `market.*`: build the feature or
  delete the schema.

## 11 — Verify ledger & test gaps

- [ ] **Repair the verify ledger** — first the cheap fixes: navigation tasks 9/11/12/13 and
  theme-system task 4 name test files that do not exist; design-revision task 14's grep never
  matched. Then write the missing tests the closed specs name: acp-client rows 10–12;
  run-supervisor rows 5–7, 20; sandbox rows 10, 11, 20, 24; pane-manager rows 14–15; telemetry
  row 9; materializers; every store row; every event-store row. A closed feature whose verify exits
  1 was never verified.
- [ ] **Prove byte-determinism for all eleven harnesses** — `assert_deterministic` covers `claude`
  only; the other ten get `assert!(result.is_ok())`. Nothing fails visibly when determinism breaks.
- [ ] **Test the 18 untested store modules** — no `#[cfg(test)]` and no integration reference for
  `agents`, `artifacts`, `bots`, `dispatch`, `handoff`, `interact`, `mail`, `memory`, `planning`,
  `projects`, `providers`, `qa`, `routing`, `runtime`, `schedules`, `session_controls`, `wiki`,
  `workflow_log`.
- [ ] **Run or delete the never-run ignored tests** — `pi_loads_generated`, `docker::connects` are
  `#[ignore]` with no `--ignored` recipe; `browse::open_contract` has an empty body.

## 12 — Model-visible resource pressure

[spec](.specs/model-resource-signal/spec.md) ·
[tasks](.specs/model-resource-signal/tasks.md)

- [ ] **Expose last-reported context occupancy and cost budgets to the agent** — freeze one short
  CTX~/BUD legend near the beginning, then keep compact `CTX~117k/200k; R~74k; N` and active
  cost-budget lines at the mutable tail. Every harness uses the same normalized ACP usage path;
  missing latest-call usage stays `CTX U`. Cost budgets are user-configured in Settings at run,
  cumulative root-task, project-day, and global-day scopes, with per-scope warning/action thresholds
  and notify/pause/cancel behavior at outer-turn boundaries. Exact values remain available through
  `locus usage --json`; the implementation follows tasks.md order.
