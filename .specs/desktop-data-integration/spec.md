# Desktop data integration

> **Navigation revision:** `planning-workspace` supersedes this spec's global project-switcher and Interact/legacy route assumptions. Its provider, envelope, live-boundary, and explicit-demo rules remain authoritative.

**Status:** planned  
**Milestone:** M2 follow-up / desktop runtime  
**Depends on:** `shell-revision`, `desktop-project-operations`, `desktop-workshop-runtime`, and the
implemented `locus-core` services they expose.

## Problem

The SolidJS desktop is data-shaped but not data-backed. Most `apps/desktop/src/data/*` accessors return
arrays from `src/fixtures`, and `App.tsx` mounts fixture screens. The existing `FixtureNotice` is unused,
so invented projects, runs, telemetry, memory, harnesses, and workshop records look durable. The test
suite largely verifies those fixtures in jsdom rather than proving a Tauri window can read and mutate the
Rust store.

The shell also contains production-visible interaction defects: the project switcher is inert, Dispatch
**Stop all** only navigates to Autorun, native decorations duplicate the custom title bar, and rail
buttons inherit browser default styling.

## Goal

Make the desktop a typed, data-driven client of the Rust core. In a real Tauri window, every product
surface reads scoped state from registered Tauri commands or `Channel` streams, renders honest loading,
empty, and error states, and sends mutations through the same boundary. Fixtures remain available only
through an explicit demo/test provider and can never silently appear in production mode.

## Scope

### In scope

1. **Data boundary.** Inventory every `Becomes: invoke`/`Channel` accessor, add the missing thin Tauri
   commands and core query/mutation methods, define typed request/response/error shapes, and register
   every command in the host.
2. **Tracer-bullet path.** Make Setup → project selection → project repos/harnesses/base context a live
   round trip against the test store before migrating the remaining screens.
3. **Screen migration.** Replace fixture reads in the shell, Inbox/Dispatch, Setup, Plan, Manage,
   Interact, Analytics/Review, Memory, and Workshop with live accessors in dependency order. A route
   without a backend renders an honest empty/not-implemented state, never invented rows.
4. **Mutations and streams.** Wire project switching, Dispatch Stop all, Inbox actions, project settings,
   plan actions, and live session/telemetry updates to Rust commands or `Channel` subscriptions with
   scoped invalidation and visible failures.
5. **Shell correctness.** Resolve the native-versus-custom title-bar ownership, implement functional
   window controls if custom chrome is retained, repair project-switcher semantics, reset rail control
   styling, and verify the shell at real window sizes in both themes.
6. **Boundary tests.** Keep pure fixture tests as provider/component tests, add static guards against
   fixture imports in production paths, test command contracts and project isolation, and add a real
   Tauri-window smoke path for loading, navigation, menu actions, and error states.

### Out of scope

- Redesigning the Rust domain schemas or replacing Postgres/test-store ownership.
- Adding new product features not represented by the current route contracts.
- Replacing SolidJS, Kobalte, CodeMirror, or the existing Tauri 2 runtime.
- Keeping fake rows as a fallback when the daemon/store is unavailable; degraded mode is an explicit
  empty/error state instead.
- Visual redesign beyond fixing the title-bar, rail-control, sizing, overflow, and theme defects
  required for the current UI contract.
- Packaging, signing, auto-update, and production distribution.

## Contracts

### Production data mode

- `isTauri()` is not a permission to fall back to fixtures. A Tauri window always uses the live provider.
- Fixture/demo data is selected explicitly by a test/demo bootstrap and is rejected when a live Tauri
  host is present.
- Screens depend on typed provider interfaces, not on `src/fixtures` and not on raw `invoke` calls
  scattered through view components.
- Every asynchronous surface has `loading`, `ready`, `empty`, and `error` states. An IPC failure is
  visible and machine-readable; it never becomes a successful fixture response.

### Scope and ownership

- Every project-scoped request carries `projectId` from the canonical locator/selected project.
- Rust validates ownership before returning or mutating rows. Cross-project identifiers return a typed
  not-found/forbidden result rather than another project's data.
- Global surfaces explicitly use global scope and do not inherit the selected project accidentally.
- Stream subscriptions are owned by the active run/session and are cleaned up when the view changes.

### UI actions

- Project selection updates the canonical navigation target and reloads project-scoped resources.
- **Stop all** calls the supervisor stop boundary, reports its result, and does not merely navigate.
- One window-chrome owner renders traffic lights/title controls; no duplicate native/custom chrome is
  allowed.
- Rail controls use shared tokens and semantic button/menu primitives rather than browser defaults.

### Testing boundary

- A passing fixture/jsdom test proves only the supplied provider and component contract.
- A passing Rust command test proves only the host/core boundary.
- The release gate requires a Tauri-window smoke test that exercises at least one live read, one scoped
  navigation, one mutation, one stream update, and one backend error. Environment-dependent checks are
  explicitly reported, never silently skipped.

## Vertical-slice order

1. **Live Setup slice:** launch Tauri → list projects from Rust → select a project → load repos and
   harness policy → render empty/error states.
2. **Live shell slice:** selected project flows through the rail, project switcher, title-bar pills, and
   locator; the Stop all command is observable.
3. **Live run slice:** Sessions/Dispatch/Inbox read the same run/event state and subscribe to changes.
4. **Live configuration slice:** Plan, Setup mutations, Workshop, and project settings persist and
   reload from Rust.
5. **Live knowledge/analytics slice:** Memory, Wiki, Artifacts, Telemetry, and Analytics query the
   scoped store and report unavailable data honestly.
6. **Real-window gate:** run the complete Tauri smoke and visual checks at the configured window size and
   one smaller supported size in Dark and Light themes.

## Success criteria

- No production screen renders `src/fixtures` data or a `*Fixture` component.
- No live-mode accessor returns a fixture after an IPC failure.
- The Setup tracer bullet displays rows created by the Rust test store, not constants from TypeScript.
- Changing the project changes only project-scoped data and the canonical locator.
- Stop all produces a supervisor mutation and visible state change/error.
- The screenshot's second title-bar/traffic-light row and washed-out rail buttons are gone.
- The desktop suite includes a real Tauri-window smoke test; fixture tests are labeled as such.
- `just test`, `just test-node`, `just typecheck`, `just lint`, and the real-window smoke gate pass.

## Decisions required before implementation

1. **Window chrome:** prefer native Tauri decorations and remove the duplicate custom traffic-light
   controls; choose custom undecorated chrome only if platform window controls are implemented and tested.
2. **Provider shape:** use one typed async resource/provider interface per data family, with a shared IPC
   error envelope; do not make each screen invent its own loading/error state.
3. **Demo mode:** keep fixtures in a test/demo provider outside production data modules, selected by an
   explicit test bootstrap rather than automatic browser/Tauri fallback.
4. **Real-window driver:** validate the repository's supported Tauri/WebDriver or equivalent test path
   before adding a dependency; a mocked `invoke` test alone does not satisfy the boundary gate.
