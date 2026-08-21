# TODO

54 features, 1055 tasks, across eleven milestones. Every task carries a runnable
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
| **M1** | Core runtime | 13 | 253 | 24 |
| **M2** | Workspace | 3 | 50 | 0 |
| **M3** | Coordination, memory, and mail | 6 | 125 | 0 |
| **M3.5** | Agent capabilities: debug and browser | 3 | 56 | 0 |
| **M4** | Workflow canvas | 3 | 65 | 0 |
| **M5** | Project management | 4 | 100 | 0 |
| **M6** | Automation and discoverability | 3 | 46 | 0 |
| **M7** | GitHub | 3 | 42 | 0 |
| **M8** | Marketplace installer | 1 | 11 | 0 |
| | | **54** | **1055** | **307** |

---

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

12 features · 265 tasks

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

## M1 — Core runtime

13 features · 253 tasks

- [ ] **[acp-client](.specs/acp-client/spec.md)** · 13 tasks · [tasks](.specs/acp-client/tasks.md)
  An Agent Client Protocol client, **for the planning/chat module only**.
  *Depends on:* `sandbox`, `telemetry`
- [ ] **[agent-cli](.specs/agent-cli/spec.md)** · 24 tasks · [tasks](.specs/agent-cli/tasks.md)
  `crates/locus-cli` — the binary agents call from inside their container, and **the MCP replacement**.
  *Depends on:* `store`, `sandbox`
- [ ] **[agent-definitions](.specs/agent-definitions/spec.md)** · 16 tasks · [tasks](.specs/agent-definitions/tasks.md)
  An agent is a Markdown file with frontmatter.
  *Depends on:* `store`, `materializers`
- [ ] **[artifacts](.specs/artifacts/spec.md)** · 21 tasks · [tasks](.specs/artifacts/tasks.md)
  What you review instead of tool calls.
  *Depends on:* `store`, `run-supervisor`
- [ ] **[ci](.specs/ci/spec.md)** · 15 tasks · [tasks](.specs/ci/tasks.md)
  Continuous integration for Locus itself, and one check that is not ordinary CI hygiene: the **materialization smoke test**.
  *Depends on:* `materializers`, `harness-registry`, `telemetry`
- [ ] **[harness-registry](.specs/harness-registry/spec.md)** · 18 tasks · **1 done** · [tasks](.specs/harness-registry/tasks.md)
  Load `harnesses/*`, validate them, and resolve a model tier into an actual model.
  *Depends on:* `store`
- [ ] **[linters](.specs/linters/spec.md)** · 14 tasks · [tasks](.specs/linters/tasks.md)
  `locus lint` — **the one extension type no harness reads.** The other seven are consumed by the harness; linters exist so that `locus lint` can find them, which is why **every harness supports linters trivially and identically** and why the registry has to say that rather than leaving the entry out.
  *Depends on:* `materializers`, `agent-cli`
- [ ] **[materializers](.specs/materializers/spec.md)** · 20 tasks · [tasks](.specs/materializers/tasks.md)
  The code half of the harness contract.
  *Depends on:* `harness-registry`
- [ ] **[pane-manager](.specs/pane-manager/spec.md)** · 21 tasks · [tasks](.specs/pane-manager/tasks.md)
  The pane manager and the IPC discipline behind it.
  *Depends on:* `app-shell`, `run-supervisor`, `telemetry`
- [ ] **[run-supervisor](.specs/run-supervisor/spec.md)** · 22 tasks · [tasks](.specs/run-supervisor/tasks.md)
  Spawn, stream, normalize, persist, cancel — and hold the session/run/turn model that everything above depends on.
  *Depends on:* `sandbox`, `materializers`, `agent-definitions`, `telemetry`
- [ ] **[sandbox](.specs/sandbox/spec.md)** · 24 tasks · [tasks](.specs/sandbox/tasks.md)
  One container per agent run, and the credential handling that makes it safe.
  *Depends on:* `spike-sandboxed-harness`, `harness-registry`
- [x] **[store](.specs/store/spec.md)** · 23 tasks · **complete** · [tasks](.specs/store/tasks.md)
  Postgres as the single source of truth, plus the backup that makes that safe.
  *Depends on:* none
- [ ] **[telemetry](.specs/telemetry/spec.md)** · 22 tasks · [tasks](.specs/telemetry/tasks.md)
  Four capture paths, one event vocabulary, and nothing downstream knowing which path a run arrived through.
  *Depends on:* `harness-registry`, `store`

## M2 — Workspace

3 features · 50 tasks

- [ ] **[editor](.specs/editor/spec.md)** · 22 tasks · [tasks](.specs/editor/tasks.md)
  CodeMirror 6 used directly, no wrapper interface.
  *Depends on:* `spike-editor-embed`, `screens-develop`, `pane-manager`
- [ ] **[lsp](.specs/lsp/spec.md)** · 17 tasks · [tasks](.specs/lsp/tasks.md)
  Semantic navigation for two consumers with one implementation.
  *Depends on:* `editor`, `sandbox`
- [ ] **[project-search](.specs/project-search/spec.md)** · 11 tasks · [tasks](.specs/project-search/tasks.md)
  Search across a project's repos — plural, because a Locus project holds one or more and the board, wiki and memory already span all of them.
  *Depends on:* `editor`, `store`

## M3 — Coordination, memory, and mail

6 features · 125 tasks

- [ ] **[guardrails](.specs/guardrails/spec.md)** · 23 tasks · [tasks](.specs/guardrails/tasks.md)
  What makes leaving a loop unattended defensible.
  *Depends on:* `run-supervisor`, `mail`
- [ ] **[handoffs](.specs/handoffs/spec.md)** · 17 tasks · [tasks](.specs/handoffs/tasks.md)
  The guardrails already kill and reassign after three stuck iterations, and a session already belongs to exactly one agent.
  *Depends on:* `mail`, `run-supervisor`, `guardrails`
- [ ] **[mail](.specs/mail/spec.md)** · 16 tasks · [tasks](.specs/mail/tasks.md)
  Agent-to-agent messages, Rust-native, identical for every harness.
  *Depends on:* `store`
- [ ] **[memory](.specs/memory/spec.md)** · 36 tasks · [tasks](.specs/memory/tasks.md)
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

3 features · 65 tasks

- [ ] **[marketplace-index](.specs/marketplace-index/spec.md)** · 15 tasks · [tasks](.specs/marketplace-index/tasks.md)
  The resolver, not the installer.
  *Depends on:* `agent-definitions`
- [ ] **[workflow-canvas](.specs/workflow-canvas/spec.md)** · 24 tasks · [tasks](.specs/workflow-canvas/tasks.md)
  Where orchestration becomes authorable, and where PLAN.md says the product's character arrives.
  *Depends on:* `spike-workflow-canvas`, `screens-workshop`, `workflow-engine`
- [ ] **[workflow-engine](.specs/workflow-engine/spec.md)** · 26 tasks · [tasks](.specs/workflow-engine/tasks.md)
  The execution half.
  *Depends on:* `guardrails`, `run-supervisor`, `sandbox`

## M5 — Project management

4 features · 100 tasks

- [ ] **[board](.specs/board/spec.md)** · 18 tasks · [tasks](.specs/board/tasks.md)
  Deliberately small.
  *Depends on:* `store`, `workflow-engine`, `screens-automate`
- [ ] **[calibration-loop](.specs/calibration-loop/spec.md)** · 20 tasks · [tasks](.specs/calibration-loop/tasks.md)
  **What makes the system improve rather than merely repeat.** A failure that only ever produces a retry teaches nothing; a failure that changes a template is paid for once.
  *Depends on:* `workflow-engine`, `planning-module`, `memory`
- [ ] **[planning-module](.specs/planning-module/spec.md)** · 40 tasks · [tasks](.specs/planning-module/tasks.md)
  A guided conversation that produces a reviewable plan.
  *Depends on:* `acp-client`, `board`, `wiki`, `screens-plan`
- [ ] **[wiki](.specs/wiki/spec.md)** · 22 tasks · [tasks](.specs/wiki/tasks.md)
  **Ingested and typed, not a blank page.** The premise taken from `llm-wiki-agent` is the right one: most knowledge tools make you search your own notes; this one reads everything you have collected and writes a structured wiki that compounds.
  *Depends on:* `store`, `screens-wiki`, `workflow-canvas`

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

## M7 — GitHub

3 features · 42 tasks

- [ ] **[agent-prs](.specs/agent-prs/spec.md)** · 14 tasks · [tasks](.specs/agent-prs/tasks.md)
  Agent-authored PRs as a **first-class flow** rather than "the agent ran `gh pr create`".
  *Depends on:* `github`, `artifacts`, `locus-browse`
- [ ] **[ci-babysitter](.specs/ci-babysitter/spec.md)** · 14 tasks · [tasks](.specs/ci-babysitter/tasks.md)
  A failing pipeline pulls its logs, feeds them to an agent, retries a bounded number of times, then escalates.
  *Depends on:* `github`, `guardrails`, `agent-prs`
- [ ] **[github](.specs/github/spec.md)** · 14 tasks · [tasks](.specs/github/tasks.md)
  Version control, CI/CD, PRs — **and Issues as an input to the Locus board.
  *Depends on:* `repo-manager`, `board`

## M8 — Marketplace installer

1 features · 11 tasks

- [ ] **[marketplace-installer](.specs/marketplace-installer/spec.md)** · 11 tasks · [tasks](.specs/marketplace-installer/tasks.md)
  The half that puts tools in images.
  *Depends on:* `marketplace-index`, `sandbox`

---

## Open decisions

54 questions the specs record as genuinely undecided, rather than answering by guess.
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
| M1 | `acp-client` | Whether the planning conversation gets a PTY-less pane type of its own or reuses the Agent Pane with the terminal suppressed. The handoff draws the Plan screen as a conversation, which suggests its own. |
| M1 | `agent-cli` | The row threshold at which key-packing engages. PLAN.md gives the technique and the saving but not the count, and below some size the header row costs more than it saves. |
| M1 | `agent-definitions` | Whether `harness: any` should resolve at run start or be pinned at save. PLAN.md does not say, and the difference only matters once one project runs more than one harness for the same agent. |
| M1 | `artifacts` | The compaction threshold. PLAN.md says "over a threshold" without naming one; it should be a setting with a defensible default rather than a constant chosen here. |
| M1 | `ci` | Whether CI runs the container-dependent tests on every push or on a schedule. Twelve harness smoke tests each starting a container is not free, and the tradeoff is real — but skipping them silently is the failure this feature exists to p… |
| M1 | `harness-registry` | **CLOSED — both verified against real binaries**, with four claims refuted. See the corrections below. One new registry-shape question falls out: `dsh` selects its model through a profile patch, not a flag, so model resolution needs a strategy that is not `[models] flag`. |
| M1 | `linters` | Whether a linter can be scoped to a path glob the way `rules` are. PLAN.md describes linters as "per directory", which reads like directory scoping, but does not say whether that is the directory the linter lives in or the directory it a… |
| M1 | `materializers` | Nothing outstanding. The five-vs-six strategy discrepancy in PLAN.md §Materializers was a stale sentence and has been corrected to six. |
| M1 | `pane-manager` | Whether pane layout persists per project or globally. PLAN.md puts pane state on the session, which suggests per session, but says nothing about the arrangement itself. |
| M1 | `run-supervisor` | Whether a session can be reassigned to a different agent without a handoff. PLAN.md says a session belongs to exactly one agent and that handoff opens a *new* session — so the answer is probably no, but it is not stated as an invariant. |
| M1 | `sandbox` | Egress policy tiers. PLAN.md puts them at the same chokepoint as credential injection; whether that holds depends on Spike 1's mechanism, so the tier names and their defaults are settled with it. |
| M1 | `store` | PLAN.md defers detailed table definitions for six of the eight schemas, keeping full tables only for `memory` and `board`. The rest get written properly with their migration — this spec does not pre-empt that, and each consuming feature'… |
| M1 | `telemetry` | `dx-telemetry` in `local-dx` has absorbed four harness dialects already and PLAN.md names its normalization pass as the reference. Whether to port its per-harness tables or rewrite them is an implementation call for task 4. |
| M2 | `editor` | Which languages get Lezer grammars at M2. The spike exercised **Rust only** — LSP is one protocol and the client is language-agnostic, so that answers the protocol question and not per-language coverage. A language is a plugin: grammar, server and root-detection declared per entry, nothing hard-coded in core. |
| M2 | `lsp` | **Semantic tokens have no implementation to import** — `@codemirror/lsp-client` has none, and PLAN.md:2095/2167 assume they exist. Whoever owns M2 writes `textDocument/semanticTokens/full`, its delta form, and the decoration layer, or the tail languages get no colour. Separately: which language servers ship in a base image by default. PLAN.md makes them marketplace entries, so the honest answer may be none — but that makes `locus lsp` unavailable until an agent asks for it, which should be a deliberate choice rat… |
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
| M7 | `github` | Rate limiting and auth for `gh` when several projects are active. PLAN.md routes service credentials through the broker, but does not say whether the GitHub token follows the same path. |
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
