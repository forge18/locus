# desktop-data-integration — tasks

| # | Vertical slice | Deps | verify |
| --- | --- | --- | --- |
| 1 | Freeze the live-data contract: inventory every desktop accessor, map it to an existing or required Rust command, define project/global scope, and define the typed loading/empty/error envelope | — | `pnpm -C apps/desktop typecheck && cargo test -p locus-tauri --lib` |
| 2 | Add the shared typed IPC/provider seam with one explicit live provider and one explicit demo/test provider; a Tauri runtime may never silently select the demo provider | 1 | `pnpm -C apps/desktop test -- data/accessors data/production-boundary && pnpm -C apps/desktop typecheck` |
| 3 | Tracer bullet: Setup loads projects, selects a project, and loads that project's repos, harness policy, and base context from the Rust test store, including loading, empty, error, and cross-project rejection states | 1, 2 | `cargo test -p locus-tauri --lib setup_live_data && pnpm -C apps/desktop test -- projects/list projects/harnesses projects/repos projects/base-context` |
| 4 | Replace shell fixture inputs with live state: project switcher selection updates the canonical locator and scoped resources; Dispatch and Inbox pills reflect live counts and records | 3 | `cargo test -p locus-tauri --lib shell_queries shell_scope && pnpm -C apps/desktop test -- shell/project-switcher shell/title-pills screens/desktop-projects` |
| 5 | Wire title-bar mutations: Dispatch **Stop all**, Inbox responses, project settings, and other visible shell actions call registered commands, update state, and render typed failures instead of navigating or no-oping | 4 | `cargo test -p locus-tauri --lib shell_mutations && pnpm -C apps/desktop test -- dispatch/stop-all inbox/gate-actions projects/extension-toggle` |
| 6 | Resolve window-chrome ownership and repair shell rendering: remove duplicate native/custom chrome, make window controls/dragging functional for the chosen owner, reset rail button styles, and verify Dark/Light at configured and smaller supported sizes | 4 | `pnpm -C apps/desktop test -- shell/visual desktop-shell shell/themes && pnpm -C apps/desktop typecheck` |
| 7 | Live run slice: Sessions, Runs, Dispatch, Inbox, Interact, and the Agent Pane read the same run/event source and update through scoped `Channel`/invalidation paths; no transcript or status fixture is used in Tauri mode | 4, 5 | `cargo test -p locus-tauri --lib run_queries event_channels && pnpm -C apps/desktop test -- sessions runs dispatch inbox panes transcript` |
| 8 | Live configuration slice: Plan, Manage, Setup mutations, Workshop plugins/extensions, agent definitions, and workflow surfaces read and persist through core commands with explicit unavailable states for missing contracts | 3, 5, 7 | `cargo test -p locus-tauri --lib configuration_commands && pnpm -C apps/desktop test -- plan projects workshop workflows agents` |
| 9 | Live knowledge/analytics slice: Memory, Wiki, Artifacts, Telemetry, Review, Analytics, and the context surfaces query the scoped store and preserve stream/error semantics | 3, 7, 8 | `cargo test -p locus-tauri --lib analytics_memory_queries && pnpm -C apps/desktop test -- memory wiki artifacts telemetry dashboard review` |
| 10 | Move fixtures behind an explicit demo/test bootstrap, remove `*Fixture` routes and fixture imports from production screens/data modules, and add a static guard that fails if they return to the live path | 2, 3, 7, 8, 9 | `pnpm -C apps/desktop test -- data/production-boundary fixtures && rg -n "from ['\"]\.\.?/fixtures | FixtureView | Memory.*Fixture" apps/desktop/src --glob '!**/demo/**' --glob '!**/test/**'` |
| 11 | Add real Tauri-window acceptance coverage: boot the host against a disposable test store, load live Setup data, switch project scope, perform Stop all, observe one stream update, and surface one backend error; report unsupported environments explicitly | 3, 5, 6, 7 | `just test-desktop-integration` |
| 12 | Run the release gate and update the UI defect ledger only after the live-window path passes; fixture/jsdom tests remain labeled as provider/component tests | 10, 11 | `just test && just test-node && just typecheck && just lint && just test-desktop-integration` |

## Order and guardrails

- Task 3 is the tracer bullet. Do not build all Rust query commands or all screens before proving one
  complete live read and project switch.
- Tasks 4–9 are vertical slices. Each must include the host command, the typed provider, the screen
  state, and the boundary test for the user action it exposes.
- Task 10 does not delete useful fixture data from component tests; it moves that data behind an
  explicit provider and makes production fixture imports fail in CI.
- Task 11 may introduce a test driver only after validating the supported Tauri/WebDriver path. A
  mocked `invoke` call is supplemental and cannot substitute for the real-window gate.
- No task may make an IPC error return a fixture, empty success, or zero-valued metric. Errors remain
  visible and typed.
- Every project-scoped query and mutation must be tested with a second project to prove isolation.
- `status: failing` is the initial state for every task in an execution tracker; flip it only after the
  task's `verify` command passes.
