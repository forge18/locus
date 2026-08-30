# TODO

**Remaining:** 49 open rows across 12 workstreams, worked top to bottom.

This is the unfinished-work index. [`PLAN.md`](PLAN.md) is the architecture authority. Each feature's
`spec.md` is its contract; its `tasks.md` is the runnable decomposition and verification source. One
row of this file is worked at a time, in the order below; folded rows live as sub-bullets of their
owning workstream so nothing is tracked twice.

## Where we are

- `chore/todo-completion` carries all completed work — 25 rows finished since the 2026-08-29
  audit; the record lives in this branch's git history, not here.
- Active workstream: **1 — desktop data integration** (tasks 1–5 of 12 done: contract frozen,
  provider seam, tracer bullet, shell live state, and title-bar mutations — Save/Archive/Rename
  hit the real store; the live run slice is next).

## 1 — Desktop data integration (the epic)

Owner of everything data-backed. [spec](.specs/desktop-data-integration/spec.md) ·
[tasks](.specs/desktop-data-integration/tasks.md) · [contract](.specs/desktop-data-integration/contract.md)
(82 commands promised, 78 missing from the host, scope per command). The rows below were separate
audit findings; each is one of the epic's slices, tracked here once:

- [ ] **Run the epic slices in tasks.md order** —
  3. ~~tracer bullet: Setup reads the real store~~ done (`projects_list`/`repos_list`/
  `local_remotes_list`/`project_setup` live, ProjectsView on envelopes, cross-project rejection
  tested both sides) ·
  4. ~~shell live state~~ done (`strip_cards`/`running_count`/`inbox_pending_count` live, pills
  and rail switcher on real counts, scope + rejection tested both sides) ·
  5. ~~title-bar mutations~~ done (`project_base_context_set`/`project_archive_set`/`project_rename`
  live with typed rejections; the Inbox-response command wiring moves with slice 7's live list) ·
  6. window-chrome ownership + rail rendering (absorbs: remove duplicate window chrome, fix rail
  control rendering) · 7. live run slice: Sessions/Runs/Dispatch/Inbox/Interact/Agent Pane (absorbs:
  wire Dispatch controls) · 8. configuration slice: Plan/Manage/Setup mutations/Workshop/agents
  (absorbs: wire Manage's New Task and TaskDetail, wire Settings → Guardrails, expose the 13 core
  service families as commands, reconcile the TypeScript types with the Rust DTOs) ·
  9. knowledge/analytics slice: Memory/Wiki/Artifacts/Telemetry/Review/Analytics (absorbs: wire
  Memory actions, build the Wiki kind filter, surface backend errors in Memory) ·
  10. demo/test bootstrap + delete fixture routes (absorbs: retire `WorkshopFixtureView`, retire
  `MemoryFixtures`, use the already-wired accessors, fix the fixture-import guard's remaining
  9 violations) · 11. real Tauri-window acceptance coverage (absorbs: add live desktop integration
  coverage) · 12. release gate.
- [ ] **Stop serving compiled-in fixtures from the host** — `artifacts_list`/`artifact_comments`
  ([lib.rs:1531](apps/desktop/src-tauri/src/lib.rs#L1531)) read a store seeded by
  `seeded_artifact_store()` with fresh ids each launch, and `agent_defs_list`/`agent_def` return
  `seeded_agent_definitions()`; neither touches Postgres. Belongs to slices 3–8 (the commands exist;
  their bodies must read the store).
- [ ] **Decide the registered-but-unused commands** — `telemetry_subscribe`, `lsp_enable_descriptor`,
  `lsp_disable_descriptor`, `detach_pane`, `repo_git_state` have no frontend caller
  (`materialization_report` and `telemetry_events_replay` now do); wire a UI or drop each.

## 2 — Runtime integrity chain

The core can materialize, clone, and start a container. The session layer it needs is in place
(ACP session establishment, telemetry persistence + replay, and daemon memory-promote routing are
done and checked off in git history). Remaining, in dependency order:

- [ ] **Wire the dispatch loop** — `Daemon::spawn_run` ([daemon.rs:281](crates/locus-core/src/runtime/daemon.rs#L281))
  has zero callers and `Store::claim_dispatchable_runs` is called only from tests; `locusd` has no
  queue poll and no Tauri command starts a run. Queued runs stay queued forever. This is the caller
  the session/telemetry work has been waiting for.
- [ ] **Run bot routines** — `Store::start_bot_run` ([bots.rs:226](crates/locus-core/src/store/bots.rs#L226))
  has zero callers; a cron routine can be enabled from the UI but never fires. Depends on the
  dispatch loop.
- [ ] **Remove the PTY from the agent run** — `spawn_at_port` still attaches `AGENT_PTY`
  ([run.rs:552-553](crates/locus-core/src/runtime/run.rs#L552-L553)); [acp-client](.specs/acp-client/spec.md)
  acceptance 7 says no PTY on any run. The ACP session now exists; this row deletes the attach, the
  `PtyStream` plumbing, and the `pty_subscribe` command, and gives the UI a `Channel<Event>`.
- [ ] **Consume the context layer from the run path** — `materialize()` calls none of
  `assemble_frozen_head`/`recall_with_settings`/`ContextBudget`/`RecitationEmitter` (commit `67bf248`
  built them; [materialize/mod.rs:6](crates/locus-core/src/harness/materialize/mod.rs#L6) only
  declares the module).
- [ ] **Make store connectivity a startup health state** — `Core::load` skips the store and
  `connected_store` connects lazily, so with `DATABASE_URL` unset every store command fails
  individually while demo-provider-backed surfaces keep answering. One health state, one indicator.
- [ ] **Test the ACP namespace claim** — [acp-client](.specs/acp-client/spec.md) acceptance 3 wants a
  test asserting the agent process's namespace; the transport tests assert only the
  `docker exec -i` argv.

## 3 — Workers consolidation

- [ ] **Resolve Interact through Workers** — [InteractView.tsx:219-227](apps/desktop/src/screens/interact/InteractView.tsx#L219-L227)
  is inert, and its disposable-session, changed-file, commit, promote, discard, and research contracts
  are not equivalent to Bots. One workstream, four folded rows:
  - build Workers on the live bot/session accessors (`BotsView` hard-codes `BOTS`/`INITIAL_ROUTINES`,
    invents `cost: "$0.42"` and a context meter, and a canned transcript);
  - connect `services/interact.rs` — `promote()`/`discard()` have no Tauri command;
    [interact-sessions](.specs/interact-sessions/spec.md) acceptance 1, 2, 4, 5, 6, 9 unmet;
  - remove the Interact hard-coded `SESSIONS` rows ([InteractView.tsx:18-42](apps/desktop/src/screens/interact/InteractView.tsx#L18-L42));
  - show cost in the Agent Pane (`showCost` never passed; `permissionPosture: "bypass"` hard-coded so
    permission/elicitation/checkpoint surfaces never render — [interact-sessions](.specs/interact-sessions/spec.md)
    acceptance 14).
  Remove the Interact route only after every retained behavior is preserved or explicitly retired.

## 4 — Planning workspace & navigation revision (gated)

These wait on approving and decomposing
[`PLANNING_WORKSPACE_PLAN.md`](PLANNING_WORKSPACE_PLAN.md); nothing here starts before that spec
pair exists.

- [ ] **Adopt the Planning Workspace contract** — update `PLAN.md`, the root feature counts, and every superseded planning/navigation contract before production code changes.
- [ ] **Replace global project navigation with page-owned scope** — remove the shell project selector (this absorbs the old "wire/remove the project switcher" defect row); install the Projects / Workers / Telemetry+Automation / Workshop rail; move route authority out of fixtures; preserve compatible deep links.
- [ ] **Persist resumable planning workspaces** — amendment/feature/project scope, lifecycle, revisions, structured resume state, planning-session linkage, live IPC, explicit deletion.
- [ ] **Plan whole projects through specs and tasks** — project brief, spec map, per-spec grilling, cross-spec audit, unified dependency graph, frozen approval revision, idempotent board materialization. (Absorbs the old "wire Plan actions / capture Plan inputs" rows — the current screen is replaced, not patched.)
- [ ] **Consolidate Bots and Interact as Workers** — route/ownership side of workstream 3.
- [ ] **Rehome secondary surfaces** — Analytics Overview under Telemetry; Autorun/Schedules/Runs stay in Dispatch; Mail into project-owned background/history; Short-term/Long-term/Artifacts/Wiki under Workshop → Knowledge.
- [ ] **Add agent capability inheritance and limits** — Workshop → Extensions → Agents: CLI Tools, Commands, Skills as `DeferToProject` or `AllowOnly`; effective capabilities are a non-escalating intersection recorded per run.

## 5 — Navigation & shell polish (ungated)

- [ ] **Give ⌘P its own search backed by `search_all`** — Shell treats `k` and `p` identically
  ([Shell.tsx:100-107](apps/desktop/src/shell/Shell.tsx#L100-L107)); `search_code`/`search_wiki`/
  `search_tasks`/`search_runs`/`unified_ranking` ([palette.rs:149](crates/locus-core/src/palette.rs#L149))
  are implemented and tested with no caller. One row: host command + keybinding +
  [command-palette](.specs/command-palette/spec.md) acceptance 2 ranking.
- [ ] **Render the category TabBar** — [TabBar.tsx](apps/desktop/src/shell/TabBar.tsx) is never
  mounted; the hand-rolled version lacks `role="tablist"`/arrow keys while Kobalte
  [ui/Tabs.tsx](apps/desktop/src/ui/Tabs.tsx) sits unused; `CATEGORY_TABS.analytics` omits Mail.
  Re-check grouping against workstream 4's rail before building.
- [ ] **Mount or delete orphaned shell components** — `RunningPill`, `Strip`, `Rail`, `TitleBar`,
  `LocatorBar`, `route-scope.ts` are exported and never rendered; `ExtensionsView.tsx` and
  `HarnessesView.tsx` are complete screens nothing routes to.
- [ ] **Mount or delete orphaned screens** — DevelopView, SearchView, AppearanceSelector,
  AvatarStylePicker, ViewerStateFamilies, DesktopPlaceholder are not in the route switch.
- [ ] **Use the Toast** — `ToastRegion` is mounted but `notify()` has zero call sites; `Card`,
  `Tabs`, `Table`, `Tooltip` are likewise unconsumed.

## 6 — Workshop & canvas

- [ ] **Wire the Workshop extension editor** — "New {type}", "Sort", item rows (`aria-selected`
  pinned to index 0), "History", "Save", "Add config key", every effort `<select>` have no handler
  ([ExtensionEditor.tsx:313-320,395-406,428,455-475,493-499](apps/desktop/src/screens/workshop/ExtensionEditor.tsx#L313-L320)).
  The mutation commands arrive with epic slice 8; wire against them, not against fixtures.
- [ ] **Wire the Workshop CLI/Providers/Workflows/Governance controls** — "Upload a CLI", "Add
  provider", "Test connection"/"Save", "Reveal"/"Replace", catalogue search, model-alias toggle,
  auth-method `Segmented`, "New workflow", "Add a guardrail"
  ([WorkshopFixtureView.tsx:150-557](apps/desktop/src/screens/workshop/WorkshopFixtureView.tsx#L150-L151)).
  These controls live on a screen epic slice 10 retires — re-scope each onto its live surface as
  slice 8 lands instead of wiring the fixture view twice.
- [ ] **Wire the workflow canvas editing loop** — palette drag has no `onDragStart`, canvas no
  `onDrop`/`onDragOver`, `SolidFlow` never given `onConnect`, "+ add clause"/guardrail stepper/toggle
  do nothing. [workflow-canvas](.specs/workflow-canvas/spec.md) acceptance 7.
- [ ] **Drive the canvas Inspector from selection and render presets as nodes** — no `onSelect`
  reaches `WorkflowCanvas`; picking the Ralph preset renders `<span>`s.
- [ ] **Make the canvas reactive** — `flowNodes`/`flowEdges` computed once, store setters discarded
  ([WorkflowCanvas.tsx:244-248](apps/desktop/src/workflow-canvas/WorkflowCanvas.tsx#L244-L248)); zoom
  readout is the fixture constant `ZOOM = "100%"`.

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
