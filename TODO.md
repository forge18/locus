# TODO

74 features, 1731 tasks, across thirteen milestones. Every task carries a runnable
`verify:`; a task with none is not a task.

**How this fits together.** [PLAN.md](PLAN.md) is the architecture and the authority.
`.specs/<feature>/spec.md` states one feature's contract and acceptance and cites the PLAN.md
section that governs it — it does not restate it. `.specs/<feature>/tasks.md` is the runnable
decomposition. This file is the index over both.

**Where to start.** The three M0 spikes gate M1. Spike 1 is the highest-risk unknown in the whole
design: if a harness cannot authenticate in a container without holding a long-lived secret, the
sandbox model changes and several M1 features are rewritten. Do not build M1 against an unanswered
Spike 1 — `.specs/sandbox/tasks.md` and `.specs/run-supervisor/tasks.md` each name which of their
tasks it can void.

M0.5 does not depend on any spike answer except Spike 2 (editor) and Spike 3 (canvas), and neither
of those blocks the shell or the other twelve screens. It can run alongside M0.

---

## Progress

| Milestone | | Features | Tasks | Done |
| --- | --- | ---: | ---: | ---: |
| **M0** | Spikes — these gate M1 · **closed** | 3 | 39 | 39 |
| **M0.5** | Desktop UI on fixtures · **closed** | 12 | 268 | 268 |
| **M0.6** | desktop reconciliation · **closed** | 6 | 195 | 195 |
| **M0.7** | Current desktop mockup reconciliation | 11 | 338 | 0 |
| **M1** | Core runtime · **closed** | 14 | 270 | 270 |
| **M1.5** | ACP agent panel and controls | 4 | 44 | 0 |
| **M2** | Workspace | 3 | 57 | 0 |
| **M3** | Coordination, memory, and mail | 6 | 133 | 0 |
| **M3.5** | Agent capabilities: debug and browser | 3 | 56 | 0 |
| **M4** | Workflow canvas | 3 | 69 | 0 |
| **M5** | Project management | 5 | 127 | 0 |
| **M6** | Automation and discoverability | 3 | 46 | 0 |
| **M7** | Forge providers | 4 | 78 | 0 |
| **M8** | Marketplace installer | 1 | 11 | 0 |
| | | **74** | **1731** | **772** |

---

## Architectural review follow-on — 2026-08-23

Cross-cutting findings from [`docs/ARCHITECTURAL_REVIEW.md`](docs/ARCHITECTURAL_REVIEW.md) — a full-pass
review of `crates/locus-core`, `crates/locus-cli`, `apps/desktop/src`, and `apps/desktop/src-tauri` against
the `tech-arch-review` skill references. Verdict: no blockers. Gaps cluster in three places — **resource
unboundedness**, **atomicity**, and **boundary leak**. Each is a design decision (control never built) unless
marked code. Severity and `path:line` live in the review doc; these are the actionable rows.

### Warnings

- [x] **[code]** · unbounded socket frame — `crates/locus-cli/src/sock.rs` rejects frames above 1 MiB before
  allocation (verified by `cargo test -p locus-cli sock::`).
- [x] **[code]** · normalized event persistence is one transaction; a conflicting later event aborts the
  batch (verified by `cargo test -p locus-core persists_each_normalized_event_with_its_run_identity_and_source_record`).
- [x] **[code]** · a failed PTY attachment stops the partially started container and retains `Queued` status
  (verified by `cargo test -p locus-core stops_the_container_when_pty_attachment_fails`).
- [x] **[code]** · agent-definition version allocation holds a transaction-scoped per-name advisory lock
  before calculating the next version (verified by `cargo test -p locus-core --no-run`).
- [x] **[code]** · `pool()` is crate-private; integration migrations use the explicit `test_pool()` hook
  (verified by `cargo test -p locus-core --no-run`).
- [x] **[code]** · materialization returns `MaterializeError::InvalidJsonKeyPath` when a JSON key path
  traverses a scalar (verified by `cargo test -p locus-core materialize --no-fail-fast`).
- [x] **[code]** · credential proxy revokes completed runs, keeps a 1,024-entry audit ring, and bounds
  upstream connection, request, and response bodies (verified by `cargo test -p locus-core --lib credential_proxy::`).
- [x] **[code]** · telemetry keeps a capped per-run journal, so `events_for` is keyed lookup rather than a
  process-lifetime O(n) scan (verified by `cargo test -p locus-core --lib services::telemetry::`).
- [x] **[code]** · `ask` delegates filing and waiting-state persistence to one atomic inbox operation
  (verified by `cargo test -p locus-core files_the_question_in_the_human_inbox_with_its_session_then_blocks`).
- [x] **[code]** · Tauri and agent-socket boundaries serialize `{ kind, message }`; CLI callers retain
  `DispatchError::Daemon` with its typed kind (verified by `cargo test -p locus-cli sock::` and
  `cargo test -p locus-tauri --lib`).
- [x] **[code]** · board, mail, memory, and wiki stubs explicitly say they are unregistered; daemon routing
  reports unavailable verbs with a typed error (verified by `cargo test -p locus-cli sock::`).
- [x] **[code]** · the socket uses the closed serde `AgentSocketVerb` enum; unknown verbs are rejected at
  frame decoding (verified by `cargo test -p locus-core --lib runtime::daemon::agent_socket`).
- [x] **[code]** · fabricated metrics — `apps/desktop/src/screens/review/RunsView.tsx` renders unavailable
  cache and spend as `unknown`; it does not derive measurements from token count (verified by
  `pnpm -C apps/desktop test -- test/runs/table.test.tsx`).
- [x] **[code]** · Plan, Agent, and Shell panes register cleanup before stream setup and detach their channel
  handlers on unmount (verified by `pnpm -C apps/desktop test -- test/plan/conversation-from-core.test.ts`).
- [x] **[code]** · dual nav resolver — `Shell.tsx` exports the canonical desktop locator mapping, and a test
  round-trips every shared view through it (verified by `pnpm -C apps/desktop test --
  test/shell/desktop-route-navigation.test.ts`).

### Suggestions

- [x] **[code]** · `store/bus.rs` contains only the Postgres transport; in-process delivery uses `crate::bus`
  (verified by `cargo test -p locus-core --lib bus::`).
- [x] **[code]** · materialization strategies expose inherent methods; the unused trait and shape test are removed
  (verified by `cargo test -p locus-core --lib materialize::`).
- [x] **[code]** · `planning.rs` retains one decomposition/approval name for each operation; forwarding
  aliases are removed (verified by `cargo test -p locus-core planning`).
- [x] **[code]** · `planning.rs` keeps `Requirement` fields private and rejects blank ids or bodies in its
  constructor (verified by `cargo test -p locus-core planning`).
- [x] **[code]** · the credential proxy uses port `44000`, outside the agent allocator range
  (verified by `cargo test -p locus-core --lib runtime::run::spawns::persisted_spawn_uses_the_reserved_port`).
- [x] **[code]** · `PortAllocator` holds a loopback `TcpListener` until release, closing the allocation/bind
  race (verified by `cargo test -p locus-core --lib sandbox::ports::`).
- [x] **[code]** · rate limiting prunes expired run keys before admitting each call
  (verified by `cargo test -p locus-core --lib sandbox::services::limits::`).
- [x] **[code]** · `ArtifactStore` is documented as a non-durable fixture seam; production bodies belong in `Store`
  (verified by `cargo test -p locus-core --lib services::artifact::`).
- [x] **[code]** · the seeded artifact adapter reports `human` and nullable timestamps instead of fabricated
  identity or empty timestamps (verified by `cargo test -p locus-tauri --lib ipc_errors_expose_a_machine_readable_kind`).
- [x] **[code]** · materialization-report tests assert report structure, not harness cardinality
  (verified by `cargo test -p locus-tauri --lib materialization_report_is_derived_from_the_core_registry`).
- [x] **[code]** · canary temporary roots use per-call UUIDs rather than a module-global counter
  (verified by `cargo test -p locus-core --lib harness::`).
- [x] **[code]** · `data/agent-defs.ts` defaults a missing generated agents extension count to zero
  (verified by `pnpm -C apps/desktop test -- test/agentdefs/materialize-footer.test.tsx`).
- [x] **[code]** · `AgentDefsView.tsx` validates unknown `frontmatter.memory` before reading `scope`
  (verified by `pnpm -C apps/desktop test -- test/agentdefs/frontmatter.test.tsx`).
- [x] **[code]** · `GuardrailsView.tsx` narrows `control.kind` through an exhaustive `switch`, without
  unchecked casts (verified by `pnpm -C apps/desktop test -- test/settings/guardrails-desktop.test.tsx`).
- [x] **[code]** · `AgentPane.tsx` / `ShellPane.tsx` register stream cleanup before subscription and render
  setup failures with `InlineError` (verified by `pnpm -C apps/desktop test --
  test/panes/agent-pane-stream-error.test.tsx`).
- [x] **[code]** · `ProjectRail.tsx` safely ignores malformed persisted expansion state during render
  (verified by `pnpm -C apps/desktop test -- test/shell/rail-expansion-persists.test.tsx`).
- [x] **[code]** · `ArtifactsView.tsx` applies comment responses only when their request token remains current
  (verified by `pnpm -C apps/desktop test -- test/artifacts/comments.test.tsx`).

Two highest-leverage fixes — the `sock.rs` frame cap (memory-exhaustion) and the credential-proxy upstream
timeout (availability) — are ordered now, before the proxy coexists with the allocator under load.

## Security review follow-on — 2026-08-23

Cross-cutting hardening from `.specs/security/REVIEW-2026-08-23.md`; decisions locked, builds pending. Each is a **design decision** (control never built) unless marked code.

- [x] **[security](.specs/security/spec.md)** · 11 tasks · **complete** · [tasks](.specs/security/tasks.md)
  - F1 (design): per-project forwarding proxy for packet-level egress; microVM rejected (not cross-platform)
  - F2 (design): trusted-by-channel context + one standing rule + override ladder (once / session / global); non-blocking
  - F3 (code, LOW): boundary redaction — raw error to host log, secret-free gist upstream, keep exact-match `redact()`
  - F4 (design): socket-boundary ownership check in `runtime/daemon.rs` before any ID-bearing verb routes

## M0 — Spikes

3 features · 39 tasks · **closed**

A spike's deliverable is the verdict in its `FINDINGS.md`, not the code that produced it. The code in
`spikes/` is throwaway and should be read that way — none of it is a head start on the feature.

**M0 closed on verdicts, not on task completion.** Enough was learned to unblock M1, so the milestone
is done. Fifteen of the thirty-nine task rows were not run, and what they would have proven is
recorded as unproven rather than as passing — each one is named in its spike's FINDINGS and carried
into the milestone that inherits it. The list of what is still unproven is under **Carried out of M0**
below; it is short and it is load-bearing.

- [x] **[spike-editor-embed](.specs/spike-editor-embed/spec.md)** · 13 tasks · [FINDINGS](spikes/02-editor-embed/FINDINGS.md) · [tasks](.specs/spike-editor-embed/tasks.md)
  Confirm the editor decision before M2 depends on it.
  **VERDICT: CodeMirror 6 stays.** A real `rust-analyzer` driven through `@codemirror/lsp-client`
  returns completion, hover with the doc comment, go-to-definition and find-references. Nothing found
  argues for VSCodium. Q2 MergeView, Q3 webviews and Q4 keyboard were not exercised — see
  [FINDINGS](spikes/02-editor-embed/FINDINGS.md) and **Carried out of M0** below.
  *Depends on:* none
- [x] **[spike-sandboxed-harness](.specs/spike-sandboxed-harness/spec.md)** · 13 tasks · [FINDINGS](spikes/01-sandboxed-harness/FINDINGS.md) · [tasks](.specs/spike-sandboxed-harness/tasks.md)
  Answer the highest-risk unknown in the design: can a harness run inside a container, authenticated, without that container ever holding a long-lived secret?
  **VERDICT: yes — the host credential proxy.** The container holds a sentinel; the real credential
  never enters it, for an API key or a sign-in token. `detect` fails the build. `/workspace` clones
  from a host bare remote with no mount and pushes back, and `main` never moves.
  Q3 events was not exercised — it needs one live model call. See **Carried out of M0**.
  *Depends on:* none
- [x] **[spike-workflow-canvas](.specs/spike-workflow-canvas/spec.md)** · 13 tasks · [FINDINGS](spikes/03-workflow-canvas/FINDINGS.md) · [tasks](.specs/spike-workflow-canvas/tasks.md)
  `solid-flow` carries the whole authoring surface for workflows and rates **Medium, not High** on Context7.
  **VERDICT: `solid-flow`.** Typed nodes and named handles work, graphs round-trip byte-identical, and
  a non-terminating loop is refused at save time naming the node. Two defects found, both three-line
  fixes. Task 12 (dagre fallback) never fires: it is conditional on tasks 3–10 failing and none did.
  *Depends on:* none

## M0.5 — Desktop UI on fixtures

12 features · 268 tasks

- [x] **[app-shell](.specs/app-shell/spec.md)** · 20 tasks · [tasks](.specs/app-shell/tasks.md)
  The four bands present on every screen: title bar, category rail, per-category tab bar, and the running-agent strip.
  *Depends on:* `design-system`, `ui-primitives`
- [x] **[design-system](.specs/design-system/spec.md)** · 19 tasks · [tasks](.specs/design-system/tasks.md)
  One visual system, defined once, that every screen reads from.
  *Depends on:* none
- [x] **[fixtures](.specs/fixtures/spec.md)** · 18 tasks · [tasks](.specs/fixtures/tasks.md)
  The whole UI is built before its backend, which is a deliberate trade with one real cost: fixture shapes get invented before the Postgres schemas exist, and every invented shape is a guess to reconcile later.
  *Depends on:* none
- [x] **[navigation](.specs/navigation/spec.md)** · 18 tasks · [tasks](.specs/navigation/tasks.md)
  The view/category/locator map, and the resolver behind it.
  *Depends on:* `app-shell`
- [x] **[screens-automate](.specs/screens-automate/spec.md)** · 27 tasks · [tasks](.specs/screens-automate/tasks.md)
  Where work is assigned and watched.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-dashboard](.specs/screens-dashboard/spec.md)** · 22 tasks · [tasks](.specs/screens-dashboard/tasks.md)
  The two views of the category that is *mine*: what I need to do, and what I need to know.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-develop](.specs/screens-develop/spec.md)** · 23 tasks · [tasks](.specs/screens-develop/tasks.md)
  The hands-on surface, and the one PLAN.md calls the **primary** editor job: reviewing what an agent changed.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-plan](.specs/screens-plan/spec.md)** · 19 tasks · [tasks](.specs/screens-plan/tasks.md)
  The planning module's surface: a guided conversation that produces a reviewable plan.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-review](.specs/screens-review/spec.md)** · 30 tasks · [tasks](.specs/screens-review/tasks.md)
  What happened, and was it any good.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-wiki](.specs/screens-wiki/spec.md)** · 20 tasks · [tasks](.specs/screens-wiki/tasks.md)
  Curated prose a human reads, derived by ingest and then cleaned up.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[screens-workshop](.specs/screens-workshop/spec.md)** · 38 tasks · [tasks](.specs/screens-workshop/tasks.md)
  Where the meta-harness lives.
  *Depends on:* `app-shell`, `navigation`, `fixtures`
- [x] **[ui-primitives](.specs/ui-primitives/spec.md)** · 14 tasks · [tasks](.specs/ui-primitives/tasks.md)
  The chrome, and only the chrome.
  *Depends on:* `design-system`

## M0.6 — desktop reconciliation

6 features · 195 tasks · **closed**

- [x] **[design-desktop](.specs/design-desktop/spec.md)** · 24 tasks · **complete** · [tasks](.specs/design-desktop/tasks.md)
  Adopt the desktop handoff without porting its HTML or JavaScript. Replace the v1 fixture contract and
  settle provider references, adapter-gated routing, project scope, plan decomposition, dispatch
  policy, and workflow Governance before implementation depends on them.
  *Depends on:* M0.5
- [x] **[theme-system](.specs/theme-system/spec.md)** · 8 tasks · **complete** · [tasks](.specs/theme-system/tasks.md)
  Ship desktop Dark and cool-neutral Light through semantic tokens and a root-theme contract; later themes
  add values and fixtures without component forks.
  *Depends on:* `design-desktop`
- [x] **[desktop-application-shell](.specs/desktop-application-shell/spec.md)** · 28 tasks · **complete** · [tasks](.specs/desktop-application-shell/tasks.md)
  Replace the v1 shell with the project-scoped rail, running popover, locator, and accessible shared
  navigation.
  *Depends on:* `design-desktop`, `theme-system`
- [x] **[desktop-project-operations](.specs/desktop-project-operations/spec.md)** · 52 tasks · **complete** · [tasks](.specs/desktop-project-operations/tasks.md)
  Make Project Settings, planning decomposition, Automate, queue policy, and safe Dispatch controls
  durable and reviewable.
  *Depends on:* `desktop-application-shell`
- [x] **[desktop-knowledge-review](.specs/desktop-knowledge-review/spec.md)** · 30 tasks · **complete** · [tasks](.specs/desktop-knowledge-review/tasks.md)
  Replace v1 Inbox, Dashboard, Develop, Review, and Wiki fixtures with the scoped desktop viewers.
  *Depends on:* `desktop-application-shell`, `theme-system`
- [x] **[desktop-workshop-runtime](.specs/desktop-workshop-runtime/spec.md)** · 53 tasks · **complete** · [tasks](.specs/desktop-workshop-runtime/tasks.md)
  Build Providers, Minisign tool verification, adapter-gated harness routing, and Workflow Governance.
  *Depends on:* `desktop-application-shell`, `theme-system`

## M0.7 — Current desktop mockup reconciliation

11 features · 338 tasks

The current desktop reference is `docs/UI mockups for PLAN.md/Locus v2.dc.html` plus
`AgentPanel.dc.html`; `docs/UI_MOCKUP_REVIEW.md` is its reviewed contract. These revision specs supersede conflicting screen contracts without rewriting M0.6's
historical completion record.

- [ ] **[analytics-revision](.specs/analytics-revision/spec.md)** · 34 tasks · [tasks](.specs/analytics-revision/tasks.md)
  The global and project-scoped Analytics overview, and its queryable Telemetry sub-tab over normalized events.
  *Depends on:* `design-revision`, `shell-revision`, `telemetry`
- [ ] **[design-revision](.specs/design-revision/spec.md)** · 14 tasks · [tasks](.specs/design-revision/tasks.md)
  The current visual authority, vocabulary, view inventory, and reconciliation decisions that every other M0.7 feature inherits.
  *Depends on:* M0.6 (`design-desktop`)
- [ ] **[dispatch-revision](.specs/dispatch-revision/spec.md)** · 42 tasks · [tasks](.specs/dispatch-revision/tasks.md)
  Dispatch's Autorun, Schedules, Runs, and Settings → Guardrails contracts over the durable queue.
  *Depends on:* `design-revision`, `shell-revision`, `setup-revision`
- [ ] **[interact-sessions](.specs/interact-sessions/spec.md)** · 24 tasks · [tasks](.specs/interact-sessions/tasks.md)
  Board-less sessions: `open`, `promoted`, and `discarded`, with commit-to-branch and discard semantics.
  *Depends on:* `design-revision`, `shell-revision`, `agent-interface`
- [ ] **[knowledge-revision](.specs/knowledge-revision/spec.md)** · 38 tasks · [tasks](.specs/knowledge-revision/tasks.md)
  Current Memory, Wiki, Artifacts, and Mail screens, including curated fact revisions and contradiction handling.
  *Depends on:* `design-revision`, `shell-revision`
- [ ] **[manage-revision](.specs/manage-revision/spec.md)** · 29 tasks · [tasks](.specs/manage-revision/tasks.md)
  Manage's Kanban, List, Graph, and Timeline views over task and session projections.
  *Depends on:* `design-revision`, `shell-revision`, `plan-revision`
- [ ] **[plan-revision](.specs/plan-revision/spec.md)** · 34 tasks · [tasks](.specs/plan-revision/tasks.md)
  The seven-stage Plan pipeline, stable requirements, and card decomposition routing controls.
  *Depends on:* `design-revision`, `shell-revision`
- [ ] **[review-qa](.specs/review-qa/spec.md)** · 23 tasks · [tasks](.specs/review-qa/tasks.md)
  Scheduled or manual QA aggregation of tests, linters, LSP diagnostics, and agent reviews.
  *Depends on:* `design-revision`, `shell-revision`
- [ ] **[setup-revision](.specs/setup-revision/spec.md)** · 28 tasks · [tasks](.specs/setup-revision/tasks.md)
  Project Settings, Persistence, and scoped Analytics with ordered harnesses, repo reassignment, and extension/tool controls.
  *Depends on:* `design-revision`, `shell-revision`
- [ ] **[shell-revision](.specs/shell-revision/spec.md)** · 32 tasks · [tasks](.specs/shell-revision/tasks.md)
  The project/cross-project rail, title-bar Dispatch and Inbox pills, locator palette, merge modal, and Inbox.
  *Depends on:* `design-revision`
- [ ] **[workshop-revision](.specs/workshop-revision/spec.md)** · 40 tasks · [tasks](.specs/workshop-revision/tasks.md)
  The shared extension editor and the Harnesses, Providers, CLI, and Workflows contracts.
  *Depends on:* `design-revision`, `shell-revision`, `setup-revision`

## M1 — Core runtime

14 features · 270 tasks

- [x] **[acp-client](.specs/acp-client/spec.md)** · 13 tasks · **complete** · [tasks](.specs/acp-client/tasks.md)
  An Agent Client Protocol client — **the only agent interface**, successor to "planning/chat module
  only". Each supported harness fronts ACP; a Locus-side mapping bridges harnesses without a native mode.
  *Depends on:* `sandbox`, `telemetry`
- [x] **[agent-cli](.specs/agent-cli/spec.md)** · 24 tasks · **complete** · [tasks](.specs/agent-cli/tasks.md)
  `crates/locus-cli` — the binary agents call from inside their container, and **the MCP replacement**.
  *Depends on:* `store`, `sandbox`
- [x] **[agent-definitions](.specs/agent-definitions/spec.md)** · 16 tasks · **complete** · [tasks](.specs/agent-definitions/tasks.md)
  An agent is a Markdown file with frontmatter.
  *Depends on:* `store`, `materializers`
- [x] **[artifacts](.specs/artifacts/spec.md)** · 21 tasks · **complete** · [tasks](.specs/artifacts/tasks.md)
  What you review instead of tool calls.
  *Depends on:* `store`, `run-supervisor`
- [x] **[ci](.specs/ci/spec.md)** · 15 tasks · **complete** · [tasks](.specs/ci/tasks.md)
  Continuous integration for Locus itself, and one check that is not ordinary CI hygiene: the **materialization smoke test**.
  *Depends on:* `materializers`, `harness-registry`, `telemetry`
- [x] **[event-store](.specs/event-store/spec.md)** · 23 tasks · **complete** · [tasks](.specs/event-store/tasks.md)
  `log.entries` is the only thing Locus writes; every other table is a fold over it. Two logs sharing one
  ordering, a synchronous in-transaction fold, and two declared carve-outs for what a model or a clock produced.
  *Depends on:* `store`
- [x] **[harness-registry](.specs/harness-registry/spec.md)** · 18 tasks · **complete** · [tasks](.specs/harness-registry/tasks.md)
  Load `harnesses/*`, validate them, and resolve a model tier into an actual model.
  *Depends on:* `store`
- [x] **[linters](.specs/linters/spec.md)** · 14 tasks · **complete** · [tasks](.specs/linters/tasks.md)
  `locus lint` — **the one extension type no harness reads.** The other seven are consumed by the harness; linters exist so that `locus lint` can find them, which is why **every harness supports linters trivially and identically** and why the registry has to say that rather than leaving the entry out.
  *Depends on:* `materializers`, `agent-cli`
- [x] **[materializers](.specs/materializers/spec.md)** · 20 tasks · **complete** · [tasks](.specs/materializers/tasks.md)
  The code half of the harness contract.
  *Depends on:* `harness-registry`
- [x] **[pane-manager](.specs/pane-manager/spec.md)** · 17 tasks · **complete** · [tasks](.specs/pane-manager/tasks.md)
  The pane manager and the IPC discipline behind it.
  *Depends on:* `app-shell`, `run-supervisor`, `telemetry`
- [x] **[run-supervisor](.specs/run-supervisor/spec.md)** · 22 tasks · **complete** · [tasks](.specs/run-supervisor/tasks.md)
  Spawn, stream, normalize, persist, cancel — and hold the session/run/turn model that everything above depends on.
  *Depends on:* `sandbox`, `materializers`, `agent-definitions`, `telemetry`
- [x] **[sandbox](.specs/sandbox/spec.md)** · 24 tasks · **complete** · [tasks](.specs/sandbox/tasks.md)
  One container per agent run, and the credential handling that makes it safe.
  *Depends on:* `spike-sandboxed-harness`, `harness-registry`
- [x] **[store](.specs/store/spec.md)** · 24 tasks · **complete** · [tasks](.specs/store/tasks.md)
  Postgres as the single source of truth, plus the backup that makes that safe.
  *Depends on:* none
- [x] **[telemetry](.specs/telemetry/spec.md)** · 19 tasks · **complete** · [tasks](.specs/telemetry/tasks.md)
  One ACP path, one event vocabulary, and an append-only transcript that is deliberately never projected.
  *Depends on:* `harness-registry`, `store`

## M1.5 — ACP agent panel and controls

4 features · 44 tasks

- [ ] **[agent-interface](.specs/agent-interface/spec.md)** · 26 tasks · [tasks](.specs/agent-interface/tasks.md)
  The ACP session surface: one stream with steering, gated approvals, research, plan, and checkpoints.
  *Depends on:* `agent-dispatch-permissions`, `agent-session-controls`, `agent-session-research`
- [ ] **[agent-session-controls](.specs/agent-session-controls/spec.md)** · 11 tasks · [tasks](.specs/agent-session-controls/tasks.md)
  ACP plan, elicitation, steering, subagent, checkpoint, replay, and posture-aware permission controls.
  *Depends on:* `acp-client`, `run-supervisor`, `telemetry`
- [ ] **[agent-session-research](.specs/agent-session-research/spec.md)** · 4 tasks · [tasks](.specs/agent-session-research/tasks.md)
  Session-scoped findings inherited from planning and promoted to memory only by review.
  *Depends on:* `artifacts`, `planning-module`, `memory`
- [ ] **[agent-dispatch-permissions](.specs/agent-dispatch-permissions/spec.md)** · 3 tasks · [tasks](.specs/agent-dispatch-permissions/tasks.md)
  The per-job bypass-default/gated-opt-in control that creates the run's immutable permission posture.
  *Depends on:* `desktop-project-operations`, `agent-session-controls`

## M2 — Workspace

3 features · 57 tasks

- [ ] **[editor](.specs/editor/spec.md)** · 22 tasks · [tasks](.specs/editor/tasks.md)
  CodeMirror 6 used directly, no wrapper interface.
  *Depends on:* `spike-editor-embed`, `screens-develop`, `pane-manager`
- [ ] **[lsp](.specs/lsp/spec.md)** · 24 tasks · [tasks](.specs/lsp/tasks.md)
  Semantic navigation for two consumers with one implementation, backed by Locus-owned language
  descriptors which projects pin and pre-provision.
  *Depends on:* `editor`, `sandbox`
- [ ] **[project-search](.specs/project-search/spec.md)** · 11 tasks · [tasks](.specs/project-search/tasks.md)
  Search across a project's repos — plural, because a Locus project holds one or more and the board, wiki and memory already span all of them.
  *Depends on:* `editor`, `store`

## M3 — Coordination, memory, and mail

6 features · 133 tasks

- [ ] **[guardrails](.specs/guardrails/spec.md)** · 23 tasks · [tasks](.specs/guardrails/tasks.md)
  What makes leaving a loop unattended defensible.
  *Depends on:* `run-supervisor`, `mail`
- [ ] **[handoffs](.specs/handoffs/spec.md)** · 17 tasks · [tasks](.specs/handoffs/tasks.md)
  The guardrails already kill and reassign after three stuck iterations, and a session already belongs to exactly one agent.
  *Depends on:* `mail`, `run-supervisor`, `guardrails`
- [ ] **[mail](.specs/mail/spec.md)** · 20 tasks · [tasks](.specs/mail/tasks.md)
  Agent-to-agent messages, Rust-native, identical for every harness.
  *Depends on:* `store`
- [ ] **[memory](.specs/memory/spec.md)** · 40 tasks · [tasks](.specs/memory/tasks.md)
  What an agent recalls: scoped facts with provenance, embeddings and decay.
  *Depends on:* `store`, `telemetry`, `materializers`
- [ ] **[repo-manager](.specs/repo-manager/spec.md)** · 17 tasks · [tasks](.specs/repo-manager/tasks.md)
  The bare local remote, per-run clones, and merge-back.
  *Depends on:* `sandbox`, `store`
- [ ] **[tool-compaction](.specs/tool-compaction/spec.md)** · 16 tasks · [tasks](.specs/tool-compaction/tasks.md)
  The cheapest token is the one that never enters context.
  *Depends on:* `materializers`, `artifacts`, `telemetry`

## M3.5 — Agent capabilities: debug and browser

3 features · 56 tasks

- [ ] **[locus-browse](.specs/locus-browse/spec.md)** · 21 tasks · [tasks](.specs/locus-browse/tasks.md)
  An agent that changed a UI can look at it.
  *Depends on:* `sandbox`, `artifacts`
- [ ] **[locus-debug](.specs/locus-debug/spec.md)** · 19 tasks · [tasks](.specs/locus-debug/tasks.md)
  A DAP client in Rust, because **agents need to debug**.
  *Depends on:* `sandbox`, `guardrails`, `marketplace-index`
- [ ] **[media-artifacts](.specs/media-artifacts/spec.md)** · 16 tasks · [tasks](.specs/media-artifacts/tasks.md)
  Two representations, because a human and a model want opposite things.
  *Depends on:* `artifacts`, `locus-browse`

## M4 — Workflow canvas

3 features · 69 tasks

- [ ] **[marketplace-index](.specs/marketplace-index/spec.md)** · 15 tasks · [tasks](.specs/marketplace-index/tasks.md)
  The resolver, not the installer.
  *Depends on:* `agent-definitions`
- [ ] **[workflow-canvas](.specs/workflow-canvas/spec.md)** · 24 tasks · [tasks](.specs/workflow-canvas/tasks.md)
  Where orchestration becomes authorable, and where PLAN.md says the product's character arrives.
  *Depends on:* `spike-workflow-canvas`, `screens-workshop`, `workflow-engine`
- [ ] **[workflow-engine](.specs/workflow-engine/spec.md)** · 30 tasks · [tasks](.specs/workflow-engine/tasks.md)
  The execution half.
  *Depends on:* `guardrails`, `run-supervisor`, `sandbox`

## M5 — Project management

5 features · 127 tasks

- [ ] **[board](.specs/board/spec.md)** · 23 tasks · [tasks](.specs/board/tasks.md)
  Deliberately small.
  *Depends on:* `store`, `workflow-engine`, `screens-automate`
- [ ] **[calibration-loop](.specs/calibration-loop/spec.md)** · 20 tasks · [tasks](.specs/calibration-loop/tasks.md)
  **What makes the system improve rather than merely repeat.** A failure that only ever produces a retry teaches nothing; a failure that changes a template is paid for once.
  *Depends on:* `workflow-engine`, `planning-module`, `memory`
- [ ] **[planning-module](.specs/planning-module/spec.md)** · 40 tasks · [tasks](.specs/planning-module/tasks.md)
  A guided conversation that produces a reviewable plan.
  *Depends on:* `acp-client`, `board`, `wiki`, `screens-plan`
- [ ] **[wiki](.specs/wiki/spec.md)** · 26 tasks · [tasks](.specs/wiki/tasks.md)
  **Ingested and typed, not a blank page.** The premise taken from `llm-wiki-agent` is the right one: most knowledge tools make you search your own notes; this one reads everything you have collected and writes a structured wiki that compounds.
  *Depends on:* `store`, `screens-wiki`, `workflow-canvas`
- [ ] **[task-orchestration](.specs/task-orchestration/spec.md)** · 18 tasks · [tasks](.specs/task-orchestration/tasks.md)
  Automate is task-centric: each task owns its workflow execution, root session, and agent run tree.
  *Depends on:* `board`, `workflow-engine`, `run-supervisor`, `screens-automate`

## M6 — Automation and discoverability

3 features · 46 tasks

- [ ] **[command-palette](.specs/command-palette/spec.md)** · 14 tasks · [tasks](.specs/command-palette/tasks.md)
  Discoverability without a second navigation system.
  *Depends on:* `navigation`, `project-search`, `wiki`, `board`
- [ ] **[dashboard-metrics](.specs/dashboard-metrics/spec.md)** · 20 tasks · [tasks](.specs/dashboard-metrics/tasks.md)
  Every metric here is **already a column**, so this is a set of queries rather than new instrumentation.
  *Depends on:* `telemetry`, `workflow-engine`, `screens-dashboard`, `screens-review`
- [ ] **[schedules](.specs/schedules/spec.md)** · 12 tasks · [tasks](.specs/schedules/tasks.md)
  cron → workflow, recorded against verify results.
  *Depends on:* `workflow-engine`, `run-supervisor`

## M7 — Forge providers

4 features · 78 tasks

- [ ] **[agent-prs](.specs/agent-prs/spec.md)** · 14 tasks · [tasks](.specs/agent-prs/tasks.md)
  Agent-authored change requests as a **first-class flow**, not an agent shelling out to a provider CLI.
  *Depends on:* `forge-providers`, `artifacts`, `locus-browse`
- [ ] **[ci-babysitter](.specs/ci-babysitter/spec.md)** · 14 tasks · [tasks](.specs/ci-babysitter/tasks.md)
  A failing pipeline pulls its logs, feeds them to an agent, retries a bounded number of times, then escalates.
  *Depends on:* `forge-providers`, `guardrails`, `agent-prs`
- [ ] **[forge-providers](.specs/forge-providers/spec.md)** · 30 tasks · [tasks](.specs/forge-providers/tasks.md)
  Provider-neutral hosted change requests, CI, review comments, and explicitly linked issues for GitHub, GitLab,
  Codeberg, Bitbucket Cloud, and Bitbucket Data Center.
  *Depends on:* `repo-manager`, `board`
- [ ] **[external-work-items](.specs/external-work-items/spec.md)** · 20 tasks · [tasks](.specs/external-work-items/tasks.md)
  Import configured tracker work into the same local task workflow, with no outbound writes before Done and a
  comment-and-resolve completion delivery afterward.
  *Depends on:* `task-orchestration`, `forge-providers`, `board`

## M8 — Marketplace installer

1 features · 11 tasks

- [ ] **[marketplace-installer](.specs/marketplace-installer/spec.md)** · 11 tasks · [tasks](.specs/marketplace-installer/tasks.md)
  The half that puts tools in images.
  *Depends on:* `marketplace-index`, `sandbox`

---

## Open decisions

56 questions the specs record as genuinely undecided, rather than answering by guess.
Each names what would settle it. They are listed here because a decision made twice in two features
is a decision that will disagree with itself.

| Milestone | Feature | Question |
| --- | --- | --- |
| M0 | `spike-editor-embed` | **Sharpened, not closed.** The mitigation PLAN.md names cannot be imported: `@codemirror/lsp-client` ships **no semantic-token support at all**. Recorded in `.specs/lsp` and `.specs/editor`; the tail-language question is now about work, not timing. |
| M0 | `spike-sandboxed-harness` | **CLOSED — yes.** Egress tiers sit at the injection chokepoint and cost almost nothing there: flipping the tier to `none` refuses the same call that returned 200, before any credential is written, and one audit row per outbound call falls out of the same code path. |
| M0 | `spike-workflow-canvas` | **CLOSED — yes.** The same `<SolidFlow>` renders a wikilink graph with only a different node component. PLAN.md's "a palette, not a subsystem" holds. One rule for `.specs/wiki`: a wikilink to an unwritten page is **not** an edge. |
| M0.5 | `app-shell` | Traffic lights are drawn for macOS. What the title bar does on Windows and Linux is undecided; nothing here blocks it, but the mockup only answers for one platform. |
| M0.5 | `design-system` | Whether a light theme is ever wanted. The handoff is dark-only and Nocturne is a dark system; nothing here forecloses it, but nothing here builds for it either. |
| M0.5 | `fixtures` | Whether long lists need virtualization at these fixture sizes. Sessions is drawn at 300 rows and Runs at 612; measuring here answers it before real data makes it urgent. |
| M0.5 | `navigation` | Whether back/forward is per window or global. PLAN.md says per window; nothing at M0.5 depends on it, and the second window does not exist until M1. |
| M0.5 | `screens-automate` | The handoff draws the Kanban columns as Ready / Building / Testing / Reviewing / Waiting For Approval / Done, while PLAN.md §The board names the second column **In Progress**. Same column, two labels — one wins, and it should be settled… |
| M0.5 | `screens-dashboard` | Whether "resolved today" should be a fixed window or a count. The handoff draws three rows and says nothing about overflow. |
| M0.5 | `screens-develop` | Whether the editor tab bar supports splits at M0.5 or only at M2 with the real editor. The handoff draws a single tab strip and does not say. |
| M0.5 | `screens-plan` | PLAN.md records the "I don't know what I want yet" entry point as **undecided** — a goal is required up front and something must turn a vague idea into one. The screen has no affordance for it, which is correct until that decision is made. |
| M0.5 | `screens-review` | Whether the Sessions and Runs tables need virtualization at 300 and 612 rows. `fixtures` measures it; this screen consumes the answer. |
| M0.5 | `screens-wiki` | Whether wiki search lives on this screen or only in the command palette. PLAN.md gives the palette global search across wiki, code, tasks and runs; the handoff draws no search field here. |
| M0.5 | `screens-workshop` | PLAN.md §Navigation lists **settings and the marketplace** as Workshop contents, but the handoff's Workshop tab bar has only three tabs and neither appears. Either they are drill-downs like agent definitions, or the tab set grows. Undeci… |
| M0.5 | `ui-primitives` | Whether long lists need `@tanstack/solid-virtual` at M0.5 or only once real row counts arrive. The Sessions table is drawn at 300 rows and Runs at 612, so fixtures can answer this rather than guessing. |
| M1 | `acp-client` | Whether the planning conversation is a distinct pane or the same event-rendered Agent Pane. **Resolved in the ACP-only revision:** with the PTY gone from the agent surface, both are events — the distinction is layout, not transport, and is now pane-manager's call. |
| M1 | `agent-cli` | The row threshold at which key-packing engages. PLAN.md gives the technique and the saving but not the count, and below some size the header row costs more than it saves. |
| M1 | `agent-definitions` | Whether `harness: any` should resolve at run start or be pinned at save. PLAN.md does not say, and the difference only matters once one project runs more than one harness for the same agent. |
| M1.5 | `agent-interface` | Exact panel variants, permission label, always-available research CLI set, checkpoint retention, workflow provenance, and agent identity remain deliberate open decisions; the ACP panel handoff supplies the fixed baseline. |
| M1 | `artifacts` | The compaction threshold. PLAN.md says "over a threshold" without naming one; it should be a setting with a defensible default rather than a constant chosen here. |
| M1 | `ci` | Whether CI runs the container-dependent tests on every push or on a schedule. Twelve harness smoke tests each starting a container is not free, and the tradeoff is real — but skipping them silently is the failure this feature exists to p… |
| M1 | `harness-registry` | **CLOSED — both verified against real binaries**, with four claims refuted. See the corrections below. One new registry-shape question falls out: `dsh` selects its model through a profile patch, not a flag, so model resolution needs a strategy that is not `[models] flag`. |
| M1 | `linters` | Whether a linter can be scoped to a path glob the way `rules` are. PLAN.md describes linters as "per directory", which reads like directory scoping, but does not say whether that is the directory the linter lives in or the directory it a… |
| M1 | `materializers` | Nothing outstanding. The five-vs-six strategy discrepancy in PLAN.md §Materializers was a stale sentence and has been corrected to six. |
| M1 | `pane-manager` | Whether pane layout persists per project or globally. PLAN.md puts pane state on the session, which suggests per session, but says nothing about the arrangement itself. |
| M1 | `run-supervisor` | Whether a session can be reassigned to a different agent without a handoff. PLAN.md says a session belongs to exactly one agent and that handoff opens a *new* session — so the answer is probably no, but it is not stated as an invariant. |
| M1 | `sandbox` | Egress policy tiers. PLAN.md puts them at the same chokepoint as credential injection; whether that holds depends on Spike 1's mechanism, so the tier names and their defaults are settled with it. **Security decision 2026-08-23: per-project forwarding proxy for packet-level egress** (F1), microVM rejected as not cross-platform. See `.specs/security/`. |
| M1 | `store` | PLAN.md defers detailed table definitions for six of the eight schemas, keeping full tables only for `memory` and `board`. The rest get written properly with their migration — this spec does not pre-empt that, and each consuming feature'… |
| M1 | `telemetry` | `dx-telemetry` in `local-dx` has absorbed four harness dialects already and PLAN.md names its normalization pass as the reference. Whether to port its per-harness tables or rewrite them is an implementation call for task 4. |
| M2 | `editor` | Which languages get Lezer grammars at M2. The spike exercised **Rust only** — LSP is one protocol and the client is language-agnostic, so that answers the protocol question and not per-language coverage. A language is an internal descriptor: grammar, server, and root-detection are declared per entry, with no core language branch. |
| M2 | `lsp` | **Semantic tokens have no implementation to import** — `@codemirror/lsp-client` has none, so M2 implements `textDocument/semanticTokens/full`, its delta form, and the decoration layer. The language catalog is internal: built-ins ship with Locus, user imports are explicit and hashed, and project activation pins then pre-provisions descriptors. |
| M2 | `project-search` | Whether `codanna` indexes on a schedule, on demand, or on git change. PLAN.md has it queried live for code structure but does not say what triggers an index. |
| M3 | `guardrails` | Whether the idle window should scale with the agent's `task_class`. A research agent reading for 90 seconds is not the same as a builder silent for 90 seconds, and 60s is a single number for both. |
| M3 | `handoffs` | Whether a handoff can cross projects. PLAN.md scopes memory as never cross-project and a session to a project, which implies no — but it is not stated for handoffs. |
| M3 | `mail` | Whether an agent can address the human directly with `locus mail send`, or only through `locus ask`. PLAN.md gives `ask` as the escalation verb and describes the inbox as the same mail system, which leaves the direct path ambiguous. |
| M3 | `memory` | PLAN.md gives the decay formula and half-lives but not the initial `importance` assignment. Measured importance needs a seed value before anything has been recalled. |
| M3 | `repo-manager` | What "syncs with it" means for a linked repo in detail — fetch on demand, on a timer, or on a filesystem watch. PLAN.md says Locus syncs with your checkout but not when. |
| M3 | `tool-compaction` | The compaction threshold, shared with `artifacts`. It should be one setting used by both, not two that drift. |
| M3.5 | `locus-browse` | Recording duration caps. PLAN.md says recordings are "capped by duration" without naming the cap, and it interacts with the 30-day media retention policy. |
| M3.5 | `locus-debug` | Adapter coverage is the standing risk PLAN.md names: the client is one implementation, but every language needs its own adapter baked per project. Node, Python and Rust are well served; the tail is not, and `locus debug` is only as broad… |
| M3.5 | `media-artifacts` | The OCR-confidence threshold for falling back to the image. It is the one number that decides between a wrong fact and a token cost, and PLAN.md gives the rule but not the value. |
| M4 | `marketplace-index` | **Curation versus selection** — a vetted catalog with quality guarantees, or an open index where manifests compete and usage data ranks them. Locus already collects the usage data, which points at selection, but PLAN.md defers the argume… |
| M4 | `workflow-canvas` | A workflow cannot re-plan itself mid-run, and PLAN.md states that cost plainly. If dynamic decomposition turns out to be needed, the fix is an agent that *authors a workflow* and submits it for goal approval — not a model in the executio… |
| M4 | `workflow-engine` | What the arbiter itself costs. It is an agent with a bounded job, so every failed iteration now pays for a classification — worth it if it saves a retry, and PLAN.md does not say what the budget is. |
| M5 | `board` | The handoff's Kanban draws column 2 as **Building**; PLAN.md names it **In Progress**. One label wins, and it is decided here since this is where the column becomes real. |
| M5 | `calibration-loop` | The confidence threshold for applying a specialization record. PLAN.md gives the rule and the reason but not the number, and it is the value that decides between accumulated wisdom and a confident wrong assumption. |
| M5 | `planning-module` | PLAN.md records the **"I don't know what I want yet" entry point as undecided**. A goal is required up front, so something has to turn a vague idea into one, and whether that is a mode of this module or a separate one is not settled. |
| M5 | `wiki` | Whether `overview` regeneration on every ingest is affordable at scale. PLAN.md says it is revised on every ingest, which is a model call per document on a page that grows. |
| M6 | `command-palette` | Ranking across four very different result kinds. A wiki page, a task, a symbol and a run are not comparable by relevance score, and PLAN.md does not say how they interleave. |
| M6 | `dashboard-metrics` | The cache-rate alert threshold. PLAN.md says "below ~80% on a long session", but "long" is undefined and a short session legitimately has a low cache rate. |
| M6 | `schedules` | Timezone and DST handling for cron expressions. PLAN.md says nothing, and it is the standard place scheduled work goes wrong twice a year. |
| M7 | `agent-prs` | What "large" means for slicing. PLAN.md gives the reason but no threshold, and the wrong one produces either one unreviewable PR or five trivial ones. |
| M7 | `ci-babysitter` | Whether the babysitter runs as an ordinary workflow or as a supervisor behavior. As a workflow it is authorable and inspectable; as a supervisor behavior it is always on. PLAN.md does not say. |
| M7 | `forge-providers` | Resolved: credentials are provider-host scoped and flow only through the credential broker; all inbound events are signed webhooks, not polling. |
| M8 | `marketplace-installer` | **Curation versus selection**, still. PLAN.md names the axis and the evidence pointing at selection, but explicitly leaves the decision to this milestone. |

---

## Carried out of M0

M0 is closed. These are the things it did **not** prove, each recorded as unproven rather than as
passing, and each owned by the milestone that inherits it. Nothing here blocks M1.

| Unproven | Owner | Why it matters |
| --- | --- | --- |
| **`usage` with real token numbers** never observed from a live run | `telemetry` | PLAN.md weights agent trust by tokens per passing run and the dashboard ranks runs by it. If the number cannot be captured, spend reads *unknown* and the dashboard cannot tell a good run from an expensive one. One live model call settles it: `spikes/01-sandboxed-harness/set-credential.sh` then `run-session.sh` |
| **A second capture source** never run — only `hooks` was exercised | `telemetry` | The four capture paths are meant to be interchangeable. `locus/base-hermes` is built and `hermes acp --check` returns OK, so `run-second.sh` is one credential away |
| **`MergeView` per-chunk revert** on a real diff | `editor` | PLAN.md calls reviewing an agent's diff the **primary** editor job; `.specs/editor` acceptance 4 enters M2 unproven |
| **All three webviews** against CodeMirror | `editor` | `.specs/editor` acceptance 7. Spike 3 proved WebKit *can* break a canvas dependency silently; the editor's tree is larger |
| **Cmd chords and IME composition** | `editor` | PLAN.md says budget the time and warns against an afternoon. Still budgeted, still unspent, and the question least suited to an automated check |
| **`dsh` and `hermes` end to end** — CLI surface verified, no session run | `harness-registry` | Their harness files are now correct on argv, model flag and hook mechanism, but neither has completed a real run |

## Found by the M0 spikes

Measured against real binaries and real browsers, not inferred. Each contradicts something PLAN.md or a
harness file currently asserts.

- [ ] **`@dschz/solid-flow`, not `solid-flow`.** PLAN.md's `/dsnchz/solid-flow` is the correct GitHub
  repo and the wrong npm name — `solid-flow` on npm is an unrelated port by a different author.
  Installing the wrong one would look like it worked.
- [ ] **`solid-flow` renders nothing on WebKit.** It calls `requestIdleCallback` unguarded and WebKit
  does not implement it. Measured on WebKit 26.5: zero nodes, zero edges, one
  `ReferenceError: Can't find variable: requestIdleCallback` and no other symptom. **WebKit is the
  engine behind WKWebView (Tauri/macOS) and WebKitGTK (Tauri/Linux)** — two of three platforms. A
  three-line polyfill fixes it completely. This is the concrete instance of the risk PLAN.md §Risks
  names, and it means a Chromium-only CI check passes while two platforms are broken.
- [ ] **`ViewportPortal` is broken in `@dschz/solid-flow@0.1.4` and fails silently.** It mounts into
  `.solid-flow__viewport-portal`, an element the version never renders, so Solid's `Portal` falls back
  to `document.body` and graph-space content lands in screen space. Read `useViewport()` instead.
- [ ] **`@codemirror/lsp-client` has no semantic-token support.** PLAN.md:2167 names LSP semantic
  tokens as the mitigation for languages with no Lezer grammar, and PLAN.md:2095 routes them over
  `Channel<T>` as though they exist. They have to be written. Recorded in `.specs/lsp` and
  `.specs/editor`.
- [ ] **`/locus/config` cannot be read-only for every harness.** PLAN.md's mount table says ro, but
  Claude Code writes its transcripts, todo state and `.claude.json` inside its config home — a ro mount
  stops it starting. Two paths: ro source at `/locus/config-ro`, copied to a writable `/locus/config`.
  The determinism that matters is preserved, because the source tree is what the prompt prefix is
  built from.
- [ ] **A host unix socket cannot be bind-mounted into a container under colima/virtiofs.**
  `operation not supported`. `/run/locus.sock` still exists in the container — a relay creates it and
  forwards to host-local TCP — but a TCP port on the host gateway is reachable by *every* container on
  the machine, where a mounted socket is reachable only by the one it was mounted into. A per-run nonce
  is load-bearing on macOS, not defence in depth. `sandbox` must carry this as a platform difference.
- [ ] **`harnesses/dsh.toml`:** `[launch] argv = []` is wrong — bare `dsh` exits with
  `--profile <name> is required`; the headless entry point is `dsh --profile headless "<task>"`.
  `[models] flag = "--model"` is wrong — no such flag exists; the model is a profile patch. A `tui`
  profile also exists, so `tui = false` is an assertion about the launch configuration, not the binary.
  The "not installed on this machine" comment is stale.
- [ ] **`harnesses/hermes.toml`:** there is no `--query-file`; the non-interactive form is
  `hermes chat --cli -Q -q "<prompt>"`, and `--cli` is required because bare `hermes chat` is
  interactive. Hooks are **shell hooks declared in `config.yaml`**, not a generated Python plugin — so
  this is `entries-in`, the same shape as claude's `settings.json`, and `materializers` has one fewer
  plugin to write. New trap: first use of a hook prompts for consent and records it in
  `shell-hooks-allowlist.json`; without `--accept-hooks` the hooks **silently never fire**, and
  telemetry reads as an agent that did nothing. hermes also ships an `acp` subcommand, so its
  `telemetry.source` is a choice rather than a constraint.

## ACP-only sweep queue — CLOSED (`[x]` list below records the executed work)

The ACP-only decision (`.specs/acp-client`) retires the PTY from the agent surface and makes ACP the
only harness interface. **The sweep has been executed** — the checkbox list below records what was
reconciled (PLAN.md, pane-manager, telemetry, run-supervisor, sandbox, all eleven harness TOMLs, and
`hermes.toml` removal). No file here is pending anymore.

- [x] **`PLAN.md`** — §Harness I/O, §One clarification, container table, §Headless, §ACP, `[launch]`,
  telemetry single-source, harness table, TOML examples.
- [x] **`.specs/pane-manager`** — Shell/PTY pane retired; `Channel<Event>` only; keyboard/xterm gone.
- [x] **`.specs/telemetry`** — four sources collapsed to `acp`; teeing retired.
- [x] **`.specs/run-supervisor`** — run = one ACP session per container; no human-terminal.
- [x] **`.specs/sandbox`** — host PTY row → ACP stdio.
- [x] **`harnesses/*.toml`** — `[telemetry].source` is `acp` for all eleven; `hermes.toml` removed;
  dependent counts (11 harnesses, 29 of 88) updated across fixtures, screens-workshop, materializers,
  linters, compact, agent-definitions.

**Recorded tension:** hermes ships an `acp` subcommand per M0, so its removal is a support decision
(drop it) rather than a technical necessity; restoring it is git-recoverable and tips counts back to
twelve.

## Conflicts between PLAN.md and the design handoff

Found while writing the specs. Recorded rather than silently resolved.

- [ ] **Board column 2 is "Building" in the handoff and "In Progress" in PLAN.md.** Same column,
  two labels. Settled in `.specs/board/tasks.md` task 1, since that is where the column becomes real.
- [ ] **Workshop's tab bar has three tabs**, but PLAN.md §Navigation lists settings and the
  marketplace among Workshop's contents and neither appears. Either they are drill-downs like agent
  definitions, or the tab set grows. Does not block M0.5.
- [ ] **No "I don't know what I want yet" entry point.** Planning requires a goal up front, so
  something has to turn a vague idea into one. PLAN.md already records this as undecided; the Plan
  screen has no affordance for it, which is correct until it is decided.

## Corrections already applied to PLAN.md

Each verified against the files rather than inferred.

- [x] "eleven harnesses" → **twelve**, in seven places. The registry table collapses `pi · omp` onto
  one row, which is how the miscount happened.
- [x] "Twenty-seven of the eighty-eight entries" → **33 of 96**, counted by `weaker_than_native`.
  Both figures are now stated as computed from the registry, never hand-maintained.
- [x] Six categories → **seven**; the Wiki row added. The "category list is closed" rule **kept**,
  its count amended.
- [x] "Five strategies … only the last needs a plugin" → **six**, and `core-driven` is last.
- [x] M0's 22-document list and 27-ADR table replaced with a pointer to `.specs/`; M0.5 inserted;
  M1's superseded UI bullet rewritten.
