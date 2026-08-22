# Locus — Architecture Plan

## Context

`~/Repos/locus` is empty except for a `dx-init` scaffold. The goal is a next-gen, multi-agent,
multi-harness IDE built with Tauri + SolidJS, taking its central idea from `~/Repos/local-dx`:
**one surface that every agent harness consumes**, so an agent, skill, or rule is defined once and
reaches every runtime.

`local-dx` proved that idea for CLI harnesses on a single machine, and paid for it in symlink
propagation, drift, and prune/remove logic — the failure it names as worst is a file that is
"present, plausible, and loaded by nobody". Locus keeps the idea and drops that cost: agents run in
disposable containers, so the harness config is **materialized fresh per run** and never propagated
into a home directory at all.

The full product is planned before product code is written. This document therefore delivers an
architecture spec set plus a phased roadmap, with three spikes gating M1.

**This document ships as `PLAN.md` at the repo root** — a living architecture doc that is committed,
reviewable, and readable by agents working in the repo, not a throwaway planning artifact. It is **the**
architecture: `.specs/` decomposes it into per-feature contracts and runnable tasks, and each spec cites
the section here that governs it rather than restating it. One source for a decision, not two that
drift. `.gitignore` already excludes the generated agent directories, so `PLAN.md` and `.specs/` are
both tracked normally.

### Where this sits in the 2026 landscape

The parallel-agent category consolidated fast. Addy Osmani's three-tier frame is the useful map:
**Tier 1** in-process subagents (Claude Code subagents, Agent Teams), **Tier 2** local orchestrators
running agents in isolated workspaces on your machine (Conductor, Superset, Emdash, Sculptor, Claude
Squad, Vibe Kanban), **Tier 3** cloud async agents (Claude Code Web, Copilot Coding Agent, Jules,
Codex Web).

**Locus is Tier 2, containing Tier 1, and deliberately not Tier 3.**

What the surveyed field does *not* have — and this is the whole bet:

> "No agent-to-agent coordination, no shared config across agents. If one agent learns the codebase's
> test conventions, the other five don't benefit." — Emdash's own stated limitation, and true of
> nearly every tool in the tier.

Every Tier 2 tool solved isolation, because git worktrees made isolation easy. Almost none solved
**coordination** or **shared learning**; the ones that gesture at it (Augment's Cosmos, "memory that
survives the session") are hosted platforms. Shared memory and agent mail in Rust, available
identically to every harness, is the thing this design has that the category does not.

**The three-part decomposition is now converged on by name.** Rotifer's *Meta-Harness Convergence*
argues that independent teams are landing on the same split — persistent context, capability
configuration, and execution environment — and that it recurs because the three have different
lifecycles: context changes continuously, capabilities change deliberately, environments are
ephemeral. Anthropic calls them Session, Harness, and Sandbox; Rotifer calls them Agent Memory, Gene,
and Binding. Locus arrived at the same three under different names, which is the useful part:

| The pattern | In Locus |
| --- | --- |
| Persistent context | the **session**, and `memory.event` behind it — *"not a context window… a queryable, persistent log of everything the agent has done"* |
| Capability configuration | the **agent definition**, versioned, plus its tool allowlist and the harness contract |
| Execution environment | the **container per run**, and the clone inside it |

Convergent naming is weak evidence for a design and strong evidence against a *different* one, so this
is worth exactly what it is: confirmation that the seams are cut where others cut them, and a warning
that anything Locus puts on the wrong side of those seams will be felt.

Two other differences worth naming, since they were arrived at independently and the field validates
both: **containers rather than worktrees alone** (Emdash had to invent `$EMDASH_PORT` injection
because worktrees leave port and service collisions unsolved; Sculptor moved to containers for the
same reason), and **workflows authored on a canvas** rather than as YAML — Microsoft Conductor is the
closest analogue and it is YAML-defined.

**The control plane displacing the editor is now the consensus, not a bet.** Cursor 3 — codenamed
Glass, shipped April 2026 by a company at $2B annualized revenue — was rebuilt from scratch to make
the agent console the primary surface and push the traditional IDE to a secondary tab. Google's
Antigravity splits Editor View and Manager Surface as coequals. Anthropic and OpenAI put the
orchestration layer outside the editor entirely. Everyone agrees the orchestration layer is the
primary surface; they disagree only on where it lives. Locus sits closest to Cursor's position: panes,
tiles, board, and canvas are the product, and the editor is a pane you can go full-screen in.

One consequence of that shift bears directly on the CodeMirror decision:

> "If the agent-first interface wins, VS Code extensions become less relevant… The assumption that VS
> Code is the center of gravity for developer tooling, an assumption that has held for nearly a
> decade, is weakening."

Cursor forked VS Code to inherit its extension ecosystem, and is now building away from it. That is
the strongest available argument that skipping VSCodium costs less than it would have two years ago.

### Decisions already made

| Fork | Decision |
| --- | --- |
| Editor | CodeMirror 6, used directly — no abstraction seam. **One editor at two zoom levels**: a side pane beside an agent, and a full-window module. No second editor, no VSCodium. |
| Debug | A **DAP client in Rust**, because agents need to debug. **No debug UI at all** — the whole surface is `locus debug` in a container, and a human debugs in their own editor. |
| Harness I/O | **ACP is the only agent-session transport.** One container hosts one ACP conversation and the UI renders its normalized events. Human terminals remain available for hand work only; they are never agent sessions. |
| `local-dx` relationship | Inspiration, not dependency. Locus owns its own registry and schema. |
| Sandbox | One container per agent run. The workspace is a **clone from a local bare remote**, not a mount. Credential handling must be easy and secure; the mechanism is Spike 1's to settle. |
| Projects | A Locus project holds **one or more repos** and owns the board, wiki, and memory across all of them. **Unrelated to GitHub Projects** — the name collision is unfortunate and means nothing. |
| Git invariant | **Locus never works in `main`/`master`. It always branches.** Reaching `main` is a human action through a PR. |
| Kanban + wiki store | Both in Postgres, including wiki revisions. |
| GitHub | Version control, CI/CD, PRs, **and Issues as an input to the Locus board**. Never GitHub Projects. |
| Marketplace | Git-backed manifest index of CLIs. No MCP servers, ever. |
| Chat | A designated harness session, not a provider API layer. Providers are configured once through OS-keychain references; Locus stores neither a credential in Postgres nor a credential in a repo. |
| Shared services | **Memory, communication, and every other cross-cutting agent capability are implemented in Rust, in `locus-core`, once. Every harness uses that one implementation.** No per-harness variants, no shell-script services, no external daemons. |
| Agents | **Markdown plus a tool list.** Frontmatter (harness, model tier, tools, skills, rules, memory scope) over a prose body. Project and workflow roles narrow that baseline; no canvas, no compile. |
| Workflows | **A visual canvas** (`solid-flow`) plus **Governance**. A workflow is a loop toward a goal; goal, guardrails, and success criteria are authored outside the graph. `verify` is required and the token budget is optional. |
| Teams | **The workflow is the team.** Its agent nodes are the roster, each node carries a role, and the edges are the dependencies. No separate `Team` entity. |
| Review surface | **Artifacts**, not transcripts. Plans, diffs, diagrams, screenshots, recordings, and a walkthrough on completion — all commentable, and a comment steers the agent that made it. |
| Harness contract | **Declaration plus materialization.** A TOML file says where each of the eight extensions goes; a **materializer** puts it there. Four strategies are generic and parameterized by that TOML; the fifth is a plugin, for the harnesses whose config is code. A harness needing code is a directory, one that does not is a file. |
| Tokens | **A design constraint, not a bill.** Prefix stability is a rule the materializer obeys, tool output is compacted before it reaches context, and every surface hands an agent a summary with a handle rather than a body. Cache rate and payload-by-tool are dashboard metrics because both are already columns. |
| Navigation | **A project-scoped rail**. Global views are Inbox, Dashboard, Projects, Dispatch, Memory, Settings, and Workshop. Plan, Develop, Automate, and Review live inside the selected project's card. A selected project scopes those views; global views retain their explicit scope. One locator scheme addresses everything. |
| Handoffs | **Ownership transfers with a payload, never a transcript.** `done`, `remaining`, `attempted`, `decisions`, `open` — the successor reads that, not the predecessor's history. Kill-and-reassign already existed; this is what it hands over. |
| Tools | **Just-in-time documentation, eager installation.** The enabled CLI catalog is baked into the image; project and workflow roles only narrow it. A one-line catalog per allowlisted tool arrives only when an agent asks for it. |
| Filesystem | **No virtual filesystem.** Docker layers and `git clone --reference` already give copy-on-write; exposing Locus state as files is the thing the store exists to stop. |
| Artifacts on disk | **Text in Postgres, media as files the row points at.** Media is stored once for you and **derived on demand for a model** — OCR before pixels, keyframes before clips. Two representations, because a human and a model want opposite things. |
| Plugins | **One manifest + one executable speaking JSON-RPC 2.0 over stdio**, in any language. Core services stay internal and are never plugins. **Data-driven: a plugin declares and returns data; the first-party UI knows how to render it.** No third-party UI code, ever. |
| Board | **Fixed columns across every project**, not configurable: Ready → In Progress → Testing → Reviewing → Waiting For Approval → Done. **`blocked` is a status, not a column.** Two gating rules only. |
| Planning | **Three agents** — interviewer, researcher, auditor — over ACP. Goal is an input, not an output. The approved spec is decomposed into board cards explicitly: spec-only, every task, or spec plus selected carve-outs. Nothing reaches the board until the single final approval. |
| Testing Locus | **Event-based.** Every run normalizes into `memory.event`, so a test is "run this, assert these events appeared" — identical across all eleven harnesses, no test-only instrumentation. |
| Permissions | **You are never prompted.** A run that stops to ask has nobody watching it. What an agent may do is *declared* — the tool allowlist on the agent definition, narrowed per role by the workflow's `Agent` node — and enforced by the container, not by the harness's own gate. |
| UI components | **Kobalte** headless primitives + **shadcn-solid** components copied into the repo + **Tailwind**. Headless, because an IDE's chrome is small and its large surfaces are all bespoke or bring their own DOM. |

### v2 desktop revision

`docs/design_handoff_locus_v2/` is the adopted desktop-design reference. Its HTML and JavaScript are
reference material only: production code remains SolidJS and Rust. It replaces the removed v1 handoff
for shell geometry, palette, screen inventory, and interaction copy.

Four product changes are architecture, not paint:

1. **Providers own credential configuration.** A provider stores an OS-keychain reference, optional
   `base_url`, verification metadata, and a curated model catalog. The broker resolves the reference
   only at the egress boundary; no API key reaches Postgres, a repository, an artifact, or an agent
   container.
2. **Dispatch is a durable queue.** Global and per-project parallelism caps, priority policy,
   preemption-at-iteration-boundary, autorun state, schedules, and Stop all are supervisor state.
   Stopping never deletes branches, artifacts, or memory; it may request a handoff before stopping a
   run.
3. **Harnesses declare adapter identity; routing is policy.** A harness is selectable only with an
   adapter and configured provider. Its default model and effort, plus the six autorouting bands, live
   in settings. The registry still owns launch, telemetry, and materialization mechanisms.
4. **Workflow authoring separates graph and governance.** The visual graph contains executable nodes;
   Governance owns goal, named guardrails, and success criteria. Workflow run state belongs to runs,
   not to the authoring surface.
5. **Custom CLI tools must be signed with Minisign.** Locus settings hold trusted public keys; a
   manifest and its binary must verify before entering the enabled catalog or an image. An unsigned or
   untrusted upload is rejected; read-only access is not a meaningful containment boundary for code
   executed during an image build.

The reconciliation contract and runnable work live in `.specs/design-v2/`. The v2 Dark theme and a cool-neutral Light theme are implemented as semantic roles under a root
`data-theme` contract in `.specs/theme-system/`, so later themes add values and fixtures rather than
component forks. These specs supersede the conflicting
v1-derived portions of the M0.5 screen contracts without changing their historical completion record.

### Deliberately deferred

Recorded so they are not rediscovered as gaps. Each was considered and set aside, not overlooked.

| Deferred | Why, and when it returns |
| --- | --- |
| **Packaging, signing, auto-update** | Single-user local tool that does not exist yet. Returns at a second machine or a second person. The one non-deferrable piece — migrations before there is data worth losing — is already M1's backup/restore. |
| **Failure and degraded-mode handling** | Docker not running, Postgres down, a registered harness not installed. Application-level exception handling, not architecture. |
| **Detailed table definitions** | The eight schemas are named; only `memory` and `board` are specified in full. The rest get written properly when their migration does, and writing them twice is waste. |
| **The question-topic checklist** | Whether planning's topic list is hand-written once or derived per project. |
| **The "I don't know what I want yet" entry point** | Planning requires a goal up front, so something has to turn a vague idea into one. Separate mode or separate module, undecided. |
| **Compiling successful runs into recipes** | A 2026 result line — Skill-DisCo, SkillRT, the LOOP engine — compiles successful agent traces into branch-free procedures, reporting savings above 90% on repeated tasks. Locus is unusually well placed for it, since every run's normalized event sequence is already stored and every workflow already has a goal and a verify. It is deferred because it is worth nothing until there are many successful runs of the same shape to compile, which is an M6 condition, not an M1 one. |
| **Where the marketplace index is hosted** | The manifest schema is settled and a local directory of them carries M4's resolver perfectly well. Hosting, pinning, and the trust model are M8's problem, with the installer that makes them matter. The axis to decide on then is **curation versus selection** — a vetted catalog with quality guarantees, or an open index where manifests compete and usage data does the ranking. Locus already collects the usage data, which points at selection, but that is an argument to have with the installer in front of us. |

### One clarification carried into the design

**No TUI, because one session belongs to one container.** A TUI harness manages several sessions inside
a single process, which hides them from Locus — it could not display, count, or own them
individually. Banning TUIs preserves a 1:1 mapping between sessions and runs. Want more sessions
in parallel? More agent runs.

- **Agent run** — one ACP conversation per container. **One session per run, always.** Every supported
  harness fronts the same ACP surface; an agent is a conversation rendered as events, not a terminal.
- **Planning / chat** — the same ACP conversation shaped for spec work and questions. Structured, not a
  terminal.

This is about observability, not rendering style. Locus spawns and owns every agent process, so every
session is visible by construction.

---

## Architecture

### Process topology

```
Tauri application  (locus)
│
├── webview — SolidJS
│     Agent conversations · Editor · Board · Wiki · Planning chat · Dashboard
│                            │ typed IPC (tauri::ipc, serde)
└── Rust core — locusd  (in-process; also runnable headless for cron/CI)
      ├── harness registry     load harnesses/*, materialize config per run,
      │                        select the telemetry adapter by source
      ├── run supervisor       spawn, stream, normalize, persist, cancel
      ├── container supervisor bollard → Docker Engine API
      ├── repo manager         local bare remote, per-run clones, branch merge-back
      ├── credential broker   keeps secrets out of containers; network policy, outbound audit
      ├── LSP supervisor       host-side language servers, multiplexed to editor panes
      ├── store                Postgres (sqlx), the single source of truth
      ├── event bus            in-process broadcast + Postgres LISTEN/NOTIFY across processes
      │                        NOTIFY carries an id only — the payload cap is 8000 bytes
      ├── shared services      memory · mail · board · wiki · telemetry · tools
      │                        one Rust implementation, identical for every harness
      ├── workflow engine      loop execution, guardrails, schedules
      └── agent socket         /run/locus.sock — bind-mounted into every agent container
```

**`locusd` outlives the window.** It runs as a background service; closing the app detaches the UI and
nothing else. Runs keep streaming into Postgres, schedules keep firing, and reopening the window
re-attaches to state that never stopped. A scheduled workflow that only fires while a window happens
to be open is not a scheduled workflow.

**Every start reconciles.** On boot the supervisor compares runs marked `running` against Docker:
container alive → re-attach its stream; container gone → close the run as `aborted`, emit the event,
and put it in the inbox. Without this a crash leaves rows that claim to be running forever, and the
dashboard slowly fills with work that ended weeks ago.

**The agent-facing surface is a CLI, not a protocol.** Inside its container an agent calls `locus …`,
which speaks to `/run/locus.sock`. This is the MCP replacement: a binary costs nothing until invoked;
a server sits in context whether used or not.

The field has moved to this position since the rule was written. Reported figures put MCP schema
overhead at 55K+ tokens consumed before an agent starts working, and the interventions that shipped
through 2026 — Anthropic's Tool Search Tool, Cloudflare's Code Mode, the MCP-code-execution pattern —
are all versions of one idea: *stop loading tool definitions you aren't using*. AWS published guidance
titled "Agent skills and CLI tools: ditch the MCP overhead". A CLI plus a docs blob loaded on demand
is the endpoint those interventions are converging toward, reached directly.

### Containers

| Container | Lifetime | Purpose |
| --- | --- | --- |
| `locus-postgres` | per machine | The store. `pgvector` + `tsvector` + window functions. |
| `locus-agent-<run_id>` | per agent run | One harness process, one session. **No TUI**; ACP over stdio from the host. |
| `locus-svc-<project>-<name>` | per project | Services the *project* needs — its own Postgres, Redis, etc., declared in the project's settings. |

Network `locus-<project>` joins a project's agents and service containers. Agents reach each other and
the project's services; they do not reach other projects.

**"Headless" means no TUI, not a terminal.** The core drives the harness over ACP — stdio, one
session per run — so there is no terminal pane to show what the process wrote; the agent renders as
normalized events instead. What `tui = false` asserts is that the harness does not multiplex several
sessions inside one process. This is deliberately single-surface: the ACP event stream. The
**registry** enforces `tui = false`, not the harness: a harness file claiming `tui = true` is refused
at registration, which is why the field is required rather than defaulted.

Mounts into an agent container:

| Path | Mode | Contents |
| --- | --- | --- |
| `/run/locus.sock` | rw | Host daemon socket |
| `/locus/config` | ro | Harness config materialized for this run |

Plus a `$LOCUS_PORT` unique to the run, and the project's setup/run/teardown scripts. **The core
allocates it from 43000–43999** and records it on the run row; the browser container reaches the app
at `http://locus-agent-<run_id>:$LOCUS_PORT` by container DNS on the project network. Allocation is
the core's because two agents on one project otherwise collide on whatever the repo's dev server
defaults to — the failure Emdash had to invent `$EMDASH_PORT` to fix.

**Project configuration lives in Locus, not in a repo.** Services, scripts, ports, and the default
verify command are project settings in the `core` schema, edited in the app. **A repo is a resource
the project holds** — Locus does not write a config file into one, because half of them are your own
checkouts and the whole point of the git model is that Locus stays out of them.

A service's credentials go through the **same broker as harness auth**: project settings hold a
reference, never the secret. One secret path, one audit trail, one place to rotate.

**The workspace is not mounted.** `/workspace` is a *clone* on the container's own filesystem — see
the git model below. **No long-lived credential lives in the container** — see Credentials.

#### Images — two layers, one cache key

**The harness binary lives in the image, never on the host.** That is what makes a run reproducible
and what keeps eleven harnesses from becoming eleven host installs. `detect` runs at *image build*,
not on your machine, and its job is to fail the build when the binary is missing rather than to find
one you already have.

| Layer | Contains | Rebuilt when |
| --- | --- | --- |
| `locus/base-<harness>:<version>` | OS, git, the harness CLI, `locus` and `locus-hook` | the harness version pins change |
| `locus/agent-<hash>` | the agent's `tools` allowlist, installed from the marketplace index | the agent's tool list or a tool's pin changes |

`<hash>` is over (base image digest, sorted tools list, resolved marketplace pins), so two agents with
the same tools share one image and a changed prose body rebuilds nothing. **Config is not a layer** —
it is materialized per run into `/locus/config`, so editing a skill never invalidates an image.

A cold build is minutes and a warm start is seconds; the first run of a new agent pays once. Auth is
injected at run start and never baked, which is the constraint Spike 1 is testing.

### No virtual filesystem

Worth answering explicitly, because it looks attractive twice and is wrong both times.

**For the workspace, the copy-on-write already exists.** Docker's storage driver is a COW filesystem —
image layers are shared and only writes allocate — and `git clone --reference` against a shared object
store means N agents on one repo do not mean N copies of its history. A FUSE layer on top would buy
deduplication that is already there, and charge a daemon, a mount lifecycle, and a macOS/Linux
behavioral difference for it.

**For Locus's own state, files were the thing being fixed.** Exposing memory, mail, and the wiki as a
mounted tree is tempting because harnesses are file-native — and it is exactly what `local-dx`, `amq`,
and `memsearch` do, which is why the plan reads them for their verb sets and refuses to link them. A
file tree cannot express scope, provenance, decay, or an ACL, and two agents writing one directory is
Hermes's documented failure. The CLI over a socket answers the same need with a query behind it.

The one honest cost of saying no: an agent that wants to `cat` something must call a command instead.
That is one tool call against a mount daemon, and the command returns the compacted form anyway.

### Adding a repo

A project holds a list of repos, each added one of two ways:

| Mode | Where the code lives | You work through |
| --- | --- | --- |
| **Linked** | `~/Repos/foo` stays yours; Locus syncs with it | your own checkout, and Locus |
| **Managed** | cloned from GitHub, lives only inside Locus | Locus only |

Board, wiki, and memory are **project-wide** and span every repo in the project. A session names the
repo it is working in. That is what makes four repos that are one system — `tapestry`, `loom-db`,
`weaver`, `texere` — share one memory instead of four that never learn from each other.

**Locus never works in `main`/`master`.** Every agent run branches. The bare local remote holds `main`
and nothing Locus does writes to it; reaching `main` is a human action through a PR. This is an
invariant, not a default, and the merge-back path enforces it.

### The git model — a local remote, not shared worktrees

Every project has a **bare local remote** on the host. An agent container **clones** from it into its
own filesystem; it works, commits, and **pushes a branch** back. You then bring the work into your
normal repo with the git you already know:

```
host: /var/lib/locus/repos/<project>.git        the bare local remote
   │  clone                          push branch
   ▼                                      ▲
container: /workspace  (container-local, not a mount)
   │
   └── you:  git fetch locus && git checkout agent/<run-id>
```

This is Sculptor's pattern, and it is better than the shared-worktree approach the rest of the field
uses, for three reasons:

- **Locus stays out of your editor, your merge tool, and your shell.** Reviewing an agent's work is
  ordinary git, not a bespoke UI you have to trust.
- **Isolation is real.** An agent cannot touch your working copy, because it does not have it. A
  bind-mounted worktree can always be escaped by a path bug; a filesystem that was never mounted
  cannot.
- **Nothing to clean up.** A finished container takes its clone with it.

Two consequences to be honest about:

- **Overlap surfaces at merge.** With independent clones, two agents *can* both edit the same file.
  Each works on its own branch and merges when it is done, so this is the conflict every team already
  has and resolves the same way — not a new problem the design has to invent machinery for.
- **Clones cost disk and time.** Mitigated by cloning `--reference` against a shared object store on
  the host, so N agents on one repo do not mean N full copies.

### Credentials

**Requirement, not mechanism: getting a harness authenticated inside a container must be easy to use
and secure.** The container should not hold a long-lived secret, and setting one up should not be a
per-project chore.

Docker's sandbox product arrived at a host proxy that injects the real credential into outbound calls
so the container only ever holds a sentinel. That is one way to satisfy the requirement, and Spike 1
is where the approach gets chosen — it is the highest-risk unknown in the design and deserves an
experiment rather than a decision made on paper.

Egress control belongs wherever that lands: per-agent network policy tiers and an audit row per
outbound call, since the same chokepoint serves both.

**Agents get no Docker socket.** A container that needs a service — the project's Postgres, a Redis —
asks the core for it: `locus svc up postgres` starts it on the project network. This avoids
Docker-in-Docker entirely and keeps the daemon socket, which is root-equivalent, away from agents.

**Threat model, named:** Simon Willison's *lethal trifecta* — private data, untrusted content, and a
channel to the outside. An agent reading a repo and browsing the web has all three. The proxy is the
third leg's chokepoint, which is why it is policy plus audit and not only credential injection.

Prompt injection is OWASP's top LLM risk and the Gemini CLI CVSS-10 supply-chain attack showed the
shape: **treat all file content as untrusted input regardless of where it came from.** Of the ten
defense layers the 2026 playbooks converge on, Locus gets most for free from decisions already made —
these are the ones that need naming so they are not skipped:

| Layer | In Locus |
| --- | --- |
| Privilege separation | The tool allowlist *is* the privilege set. A reviewer agent gets read-only tools and cannot write |
| Sandboxing | Container per run, no host filesystem, no Docker socket, clone not mount |
| Egress control | The proxy, with per-agent policy tiers |
| **Rate limiting** | Cap tool calls per run, not only iterations and tokens — injection usually needs volume |
| **Canary tokens** | A secret string in the materialized config that must never appear in output. If it does, the system prompt leaked. Cheap, and detects extraction regardless of technique |
| **Anomaly detection** | Every run's tool sequence is already normalized in Postgres, so "this run's tool pattern is unlike this agent's baseline" is a query, not new instrumentation |
| Human-in-the-loop | `Gate` nodes and `locus ask`. Calibrated the way the playbooks advise: gate what is **irreversible, touches production, or reaches credentials**; let routine reads and sandboxed computation run free. Gating everything makes an agent useless; gating nothing is the whole problem |

**Stated honestly:** containers share the host kernel, so this is weaker isolation than a microVM.
Docker moved its sandbox product to microVMs for exactly that reason. Containers are the pragmatic
choice — Linux and macOS, no proprietary dependency, and the same primitive already used for project
services — but the boundary is a kernel boundary, not a hypervisor one.

### The one surface

**Eight extension types, the same set for every harness.** An extension is authored **once**, in
Locus. At run start the core reads `harnesses/<name>.toml` and writes it into the path and file
format that harness expects, under `/locus/config`, for that run only.

| Extension | Is | Consumed by |
| --- | --- | --- |
| `agents` | agent definitions — frontmatter over prose | the harness |
| `commands` | invocable prompt templates, worth writing only when they take arguments | the harness |
| `hooks` | shell scripts declaring their events, JSON in and out, exit 0 always | the harness |
| `linters` | a check plus the rule saying why, per directory | **`locus lint`** — not the harness |
| `output-styles` | how the agent writes; one is active per harness | the harness |
| `rules` | path-scoped instructions, loaded when a matching file is touched | the harness |
| `skills` | `SKILL.md` per directory, model-invocable | the harness |
| `base-context` | the always-loaded instructions every session starts with | the harness |

Two of the eight do not behave like the rest, and the registry has to say so per harness:

- **`linters` are tool-facing, not harness-facing.** No harness reads them. The directory exists so
  `locus lint` can find it, which means every harness supports linters trivially and identically.
- **`output-styles` is the most divergent.** Some harnesses have a real mechanism; others have no
  concept of it and the style has to be folded into `base-context` instead. Where that happens the
  registry records it, because a style silently merged into context is a different thing from a style
  the harness selects — and the entry should say which one you got.

`base-context` is near-uniform in practice — `AGENTS.md` almost everywhere, with `CLAUDE.md` and
`AGENTS.override.md` as the two exceptions — which is why it also serves as the **fallback injection
path** for the four harnesses that cannot inject at session start.

This inverts `local-dx`'s hardest problem. There is no propagation, no symlink graph, no `--prune`,
no drift: the target filesystem is destroyed when the run ends.

Three rules carried over verbatim from `local-dx`, because each earned its place:

1. **Nothing in core names a harness.** Adding one is a TOML file, plus a materializer plugin and a
   stream-format module only where that harness needs code — and neither is compiled into core.
2. **Every entry is complete — nothing inherited.** A harness file states every path and format it
   uses, so it reads cold.
3. **The database is derived where it can be.** Harness output, git, and the marketplace index are
   sources of truth and are never written by Locus. Board, wiki, **memory**, and **mail** are the
   exception — they exist only in Postgres, and are backed up rather than rebuilt. Backup is
   therefore a real requirement, not an operational afterthought.

### Shared services — one Rust implementation, every harness

Memory, communication, board, wiki, telemetry, and tool access are **agent capabilities, not harness
features**. Each is written once in `locus-core` and reaches every harness through the same `locus`
CLI over `/run/locus.sock`.

```
crates/locus-core/src/
  memory/      recall, write, embed, decay, scope   ← Rust, one implementation
  mail/        threads, delivery, wait, drain       ← Rust, one implementation
  board/       tasks, transitions, dependencies
  wiki/        pages, revisions, search
  telemetry/   normalized events, aggregates
  tools/       marketplace resolution, allowlist
```

The rule this enforces: **a capability never gets a per-harness implementation.** A harness supplies
model access and a tool loop; everything an agent remembers, everything it says to another agent, and
everything it records goes through Locus. Two consequences worth naming:

- Memory and mail are **queryable and dashboardable** because they are rows, not files scattered
  across harness home directories — which is exactly what made `local-dx`'s telemetry hard.
- A harness swapped mid-project keeps its memory and its inbox. Nothing is stored in a harness's own
  format, so nothing is lost when the harness changes.

**Memory is not the wiki.** The wiki is curated prose a human reads and reviews. Memory is what an
agent recalls: scoped facts with provenance, embeddings, and decay. They share `pgvector` and nothing
else.

`amq` and `memsearch` are **inspiration only** — read for their verb sets and scoping model, never
linked or shelled out to. Both would put the data back in files.

### ACP — the only agent interface

**The [Agent Client Protocol](https://agentclientprotocol.com) client is the only interface an agent
speaks to Locus over.** ACP is JSON-RPC 2.0 over stdio — LSP-for-agents — created by ZED in August 2025
and co-maintained with JetBrains. It fits a conversation: `session/new`, `session/prompt`, streamed
updates, tool-call permission requests. That vocabulary is now the vocabulary every agent pane speaks.

- Rust crates `agent-client-protocol` and `agent-client-protocol-schema` are on crates.io and current.
- Agents speaking it: **Claude Agent**, **Codex CLI**, **Cline**, **Cursor**, **Gemini CLI**,
  **Goose**, **Factory Droid**, Docker's **cagent**, **GitHub Copilot** (preview), and more.

**ACP is how agent sessions run.** Agents run as ACP conversations, one per run. The first reason
that kept ACP out of the working loop — `session/new` accepts only `cwd` and `mcpServers`, so **a
client cannot inject a system prompt or any instructions** — is kept, not erased: prompt assembly
belongs entirely to the harness, and base-context, skills, rules, and tools reach the agent through the
materialized config tree, not through the ACP session. The second reason — that a conversation
abstraction hides the thing terminals exist to show, the agent working — is retired with the PTY: an
agent is an event stream, and the pane renders those events, never a terminal.

`mcpServers` is passed empty. Always.

**Every agent runs in a container.** ACP is stdio, and stdio attaches to a container process as
readily as to a host one. Running agents on the host would give them your real filesystem and your
real credentials, which is precisely the exposure the container model exists to remove. ACP is not a
reason to leave the sandbox.

A harness is described by one TOML file. ACP does not replace the config contract — it standardises
the conversation loop, and says nothing about where a harness reads its skills, rules, or context.
That knowledge is per-harness whether the harness speaks ACP or not.

**Every supported harness fronts the ACP surface.** `[telemetry].source` collapses to `acp`. Where a
harness has no native ACP mode, a Locus-side mapping (materializer/plugin) bridges it. All sources
normalize into the one event vocabulary below, and nothing downstream knows which mapping a run
arrived through — because there is one mapping for every ACP harness, not one per harness.

**Registered harnesses.** One entry each in `harnesses/` — a file where declaration is enough, a
directory where the harness needs a materializer plugin. Every entry complete, nothing inherited:

| Harness | Binary | ACP surface | Session injection reaches it by |
| --- | --- | --- | --- |
| claude | `claude` | native | its own `SessionStart` hook |
| codex | `codex` | native | its own `SessionStart` hook |
| copilot | `copilot` | native | its own `sessionStart` hook |
| pi · omp | `pi` · `omp` | native, via a generated TS extension | the generated extension |
| **gemini** | `gemini` | native | its own hook |
| **cursor** | `agent` | **ACP** (`agent acp`) | `[layout].context` |
| **antigravity** | `agy` | **ACP** | `[layout].context` |
| **aider** | `aider` | **ACP** | `--read`, a context file |
| dsh | `dsh` | native | `[layout].context` |
| opencode | `opencode` | **ACP** | `[layout].context` |

`[telemetry].source` is real but single-valued in the ACP-only model: for every supported harness it
is `acp`. The four-source table that once selected a path per harness is retired; there is one path.
Everything still normalizes into the one event vocabulary below, and nothing downstream re-diverges.

**Every harness has every capability. Only the mechanism differs.** This is `local-dx`'s rule and it
carries over unchanged: where a harness has no native mechanism, a loader bridges it, and the harness
file records *how* rather than *whether*. There is no capability matrix and no feature flag to branch
on, because a flag would let a capability silently not arrive — the failure `local-dx` names as worst,
a file present, plausible, and loaded by nobody.

So the four that cannot inject at session start still get injection: it lands in `[layout].context`,
which every harness reads. That fallback is why the layout table survives everything else. The same
holds for the rest — an agent with no native subagent mechanism gets `locus agent invoke`, and resume
is Locus's, not the harness's: a new run is primed from the event store whether or not the harness has
a native session id to hand back.

**The harness file declares mechanism, never policy.** No `[capabilities]` block, and **no model
routing** — tier resolution is the app's, so changing which model `high` means is a setting rather
than eleven file edits.

#### Model routing — mechanism in the file, policy in the UI

An agent asks for `model_tier: high`. Turning that into an actual model takes two things the harness
file must supply, because both are facts about the binary rather than preferences about the work:

```toml
[models]
flag      = "--model"                 # how a model is passed to THIS harness
list_argv = ["models", "list"]        # optional: how to ask it what it has
```

Everything else is a setting. **Settings → Harnesses** shows one row per registered harness with four
cells — `low`, `medium`, `high`, `xhigh` — each a combobox over whatever `list_argv` returned, free
text where a harness cannot enumerate. The mapping lives in `core.settings`, keyed by harness and
tier, and changing which model `high` means is one edit that every agent picks up on its next run.

Three rules make it safe to leave half-filled:

- **A missing tier falls back UP, never down.** An agent asking for `xhigh` on a harness with no
  `xhigh` gets `high`. Falling *down* would quietly answer a hard question with a cheap model, and the
  result would look like a bad agent rather than a bad setting.
- **Unset means the harness's default.** With no mapping at all Locus passes no `flag`, so a harness
  that was never configured still runs — it just runs on whatever it would have chosen itself. A new
  harness is usable the moment it is registered.
- **The resolved model is recorded on the run.** Not the tier — the actual model id, on every run row,
  so spend and verify pass rate are attributable to what really answered. Comparing tiers across a
  setting change is otherwise guesswork.

`list_argv` is discovery, not policy: it asks the harness what exists. Which of those is `high` is
never the file's business, and a harness that gained a model overnight needs no file edit for it to
appear in the combobox.

```toml
# harnesses/claude.toml
name    = "claude"
binary  = "claude"
detect  = ["--version"]

[launch]                              # how to start ONE session in ONE run
argv    = ["--permission-mode", "bypassPermissions"]   # the harness's own gate is OFF:
                                      # the container is the boundary, and there is
                                      # nobody attached to answer a prompt.
tui     = false                       # REQUIRED false. A TUI multiplexes sessions
                                      # inside one process and hides them from Locus.

[telemetry]                           # where structured events come from
source  = "acp"                      # acp is the only source — the ACP surface
log_dir = "~/.claude/projects"        # transcripts, for backfill
format  = "claude-jsonl"

[layout]                              # all eight, and where THIS harness reads each
agents        = { dir  = "/locus/config/agents",        format = "markdown+frontmatter" }
commands      = { dir  = "/locus/config/commands",      format = "markdown+frontmatter" }
hooks         = { dir  = "/locus/config/hooks",         format = "shell" }
linters       = { dir  = "/locus/config/linters",       format = "shell+markdown" }
output-styles = { dir  = "/locus/config/output-styles", format = "markdown+frontmatter", active = "brief-bright-gone" }
rules         = { dir  = "/locus/config/rules",         format = "markdown+frontmatter" }
skills        = { dir  = "/locus/config/skills",        format = "markdown+frontmatter" }
context       = { file = "/locus/config/CLAUDE.md" }   # the `base-context` extension

# When a harness has no mechanism for an extension, say so rather than omitting it:
#   output-styles = { merged-into = "context" }

```

No `[capabilities]`, no `[model_routing]`: what the harness *can* do is universal, and which model a
tier resolves to is set in the app.

**Telemetry does not come from a terminal.** There is no terminal on the agent path; an agent renders
as normalized events, never as raw bytes. Structured events come from the harness's own ACP stream —
normalized into one vocabulary. `dx-telemetry` in `local-dx` proved the normalization pattern across
four harness dialects; ACP gives Locus one richer, structured source instead of four scraped ones.

### Materializers — the code half of the contract

Declaring *where* an extension goes is not enough. A harness with no rules directory needs its rules
turned into something it does read, and that is a transformation, not a path. `local-dx` needed a
loader script per harness per extension for exactly this reason, and Locus needs the same work done —
it just gets to do it into a scratch directory instead of into your home.

**Materializing into a fresh tree removes most of the work, not all of it.** `local-dx` wrote into
`~/.codex/AGENTS.override.md`, a file you also own, so every loader carried markers, idempotency, and
a prune path. Locus builds the whole config tree per run and throws it away after, so it **generates
whole files**. No markers, no merge, no prune, nothing to reconcile — the same inversion the design
already claims for propagation, applied to the loaders.

What survives is real: format conversion and code generation. Six strategies cover it, and only one
of them needs a plugin:

| Strategy | Does | Used by |
| --- | --- | --- |
| `dir` | copy the extension's files as they are, optionally renaming | claude, pi, omp, opencode skills; copilot agents (`suffix = ".agent.md"`) |
| `merged-into` | render the files into one target file as prose, frontmatter stripped | codex and copilot rules and output-styles; dsh commands |
| `listed-in` | write the files' paths into a key of the harness's config | opencode rules and styles → `opencode.json:instructions` |
| `entries-in` | convert each file into one structured entry in a config file | codex agents → TOML; dsh agents → `cordis.patch.yml`; claude hooks → `settings.json` |
| `plugin` | **run an executable that returns the files to write** | pi and omp hooks and rules (TS extension); opencode hooks (plugin) |
| `core-driven` | Locus fires the extension itself at the boundaries it owns | aider and cursor hooks — neither has a hook mechanism, so `session_start` and `session_end` come from the container's own lifetime |

**Every entry that is weaker than native says so.** A rule folded into always-on context is not the
same thing as a rule the harness loads when a matching file is touched, and an entry that hides the
difference is how you end up believing in a capability you do not have. So a downgraded strategy
carries `weaker_than_native` naming the loss:

```toml
rules = { via = "merged-into", target = "context", strip_frontmatter = true,
          weaker_than_native = "always-on; path scoping is lost" }
```

Thirty-three of the entries across the eleven harnesses are downgrades. That number is
the honest measure of how uneven the field is, and it is only visible because the files say it.

**Both figures are computed from the registry, never maintained by hand** — eleven harnesses times
eight extensions is the denominator, and the numerator is a count of `weaker_than_native` keys. Any
surface that shows them reads the files, so registering a harness moves the number without an edit.

The first four are parameterized data — `format`, `suffix`, `flat`, `strip_frontmatter`, the target
key — and live in `crates/locus-core/src/materialize/` as generic implementations that name no
harness. `core-driven` is generic too: it fires the extension from the container's own lifetime rather
than writing a file at all, which is what the two harnesses with no hook mechanism get.

**Materialization is byte-deterministic**, which is a token decision more than a tidiness one. Sorted
file order, sorted lists inside generated files, no timestamps, no run id, no hostname: the same agent
with the same tools must produce a byte-identical tree, because that tree *is* the prompt prefix and
an unstable prefix costs cache on every run that follows. A materializer that embeds the current time
is not slightly untidy — it is a per-run cache miss for every agent that harness serves.

**The plugin contract, for the fifth.** Same shape as every other plugin here: one executable,
JSON-RPC 2.0 over stdio, any language. It is called once per run with the extension set as JSON and
**returns the files to write** — it never writes them itself:

```
→ materialize { harness, extension, root: "/locus/config", entries: [{name, frontmatter, body}] }
← { files: [{ path, mode, content }] }
```

Core writes the returned files after checking every path resolves under `root`. Three things fall out
of returning data rather than writing it: a materializer is a pure function and therefore testable
without a container, a buggy one cannot escape the config tree, and the same JSON is the fixture for
"did this harness get its rules" as an event-based test.

**A harness that needs code is a directory; one that does not is a file.**

```
harnesses/
  claude.toml            pure data — every extension is native
  pi/
    pi.toml
    materialize          executable; generates the TS extension pi reads
```

**Canonical event vocabulary** — every source normalizes to exactly this set:

```
session_start  user  assistant  thinking  tool_call  tool_result  tool_error
permission_request  subagent_start  subagent_stop  aborted  session_end
```

Every event carries `run_id`, `seq`, `ts`, and a `raw` JSONB of the source record it was built from,
so a normalization bug is repairable by replay rather than by re-running the agent.

**Token usage is an attribute, not a verb.** `assistant` and `session_end` carry
`usage { input, output, cache_read, cache_write }` exactly as the harness reports it — the harness gets
the number from the model API, and Locus never counts tokens itself. Where a harness reports nothing,
`usage` is null and spend reads *unknown* rather than zero.

**`permission_request` is a misconfiguration alarm.** Since every harness launches with its own gate
off, one firing means a gate was left on and the run is about to hang. It stays in the vocabulary for
exactly that reason — see Permissions below.

**How each source normalizes.** `[telemetry].source` is `acp` for every supported harness; the module
under `crates/locus-core/src/telemetry/` is one ACP path, and nothing downstream knows which one:

| Source | How it arrives | Mapped by |
| --- | --- | --- |
| `acp` | `session/update` notifications on the stream the ACP client already holds | `AgentMessageChunk` → `assistant`, `AgentThoughtChunk` → `thinking`, `ToolCall`/`ToolCallUpdate` → `tool_call`/`tool_result`/`tool_error` by its `status`, `RequestPermission` → `permission_request`. **One mapping for every ACP harness**, not one per harness |

**The four-source table is retired.** `hooks`, `stream-json`, and `session-log` no longer feed the
agent surface; the richest and the weakest both go. A harness with no native ACP mode is bridged by a
mapping, not dropped to a terminal capture — there is no terminal capture to fall back to.

**Teeing stdout has no counterpart here.** `stream-json` teeing existed to mirror structured bytes to
a terminal; there is no terminal. The single source is the ACP stream, nothing else.

Three rules keep the one path honest:

- **Ordering is Locus's.** `seq` is assigned on arrival at the core, so a source with no ordering
  guarantee still yields a totally ordered stream.
- **A missing verb is recorded as missing, never synthesized.** A harness that cannot report `thinking`
  produces no `thinking` events rather than empty ones. Each `harnesses/*.toml` declares the verb set
  its source can emit, so an event-based test knows what to expect per harness — otherwise every
  assertion would have to be written to the weakest path.
- **`raw` is kept on every event.** Harness formats change between releases; replay against a fixed
  parser is the repair, and it is the reason capture is separated from normalization at all.

**What a harness must supply to be supported:** a launch command, a known config layout, and an ACP
telemetry source. A harness that only paints a TUI is unsupported — not because of its output format,
but because it breaks the one-session-per-run mapping everything else depends on.

**Why Tauri, restated for this model.** The usual argument for Electron is that an in-process Node
runtime is free when the IDE drives agents through TypeScript SDKs. Locus drives agents as **subprocesses
that answer one ACP conversation each** — so no in-process language runtime is needed on either side of
that choice, and the argument evaporates. What Locus does need is container lifecycle, ACP over stdio,
Postgres, and git, all first-class in Rust. **Tauri's real cost is webview inconsistency**
(WKWebView / WebView2 / WebKitGTK); agent-terminal keyboard fidelity falls away with ACP.

### Data model — Postgres schemas

| Schema | Holds |
| --- | --- |
| `core` | projects, repos (multi-repo per project), local remotes, settings |
| `agents` | `agent_defs` (versioned; frontmatter JSONB + Markdown body), sessions, runs, parent/child run edges, normalized events, **artifacts and their comment threads** — an artifact row carries kind, text body or blob path, media type, `sha256`, and the derived-representation cache |
| `board` | tasks (fixed columns, blocked as a status), **dependency edges**, transitions, assignments, task↔run links, evidence links, **linked GitHub issues** |
| `wiki` | pages (**typed by kind**), revisions, links, contradictions, ingest log, embeddings (`pgvector`) |
| `memory` | **core** (bounded, per-agent and per-project, materialized per run) and **store** (facts, scope, provenance, embeddings, confidence, decay) |
| `workflows` | `workflow_defs` (versioned; `graph` + `spec` JSONB), schedules, executions, iterations, guardrail trips, verify results |
| `mail` | threads, messages, delivery state |
| `market` | manifests, installs, per-image tool sets |
| `log` | `entries` — Locus's domain event log; the only table any of the above is written *from* |

### Event sourcing and its two carve-outs

**`log.entries` is the only thing Locus writes; every other table is a fold over it.** A row is a
cached answer, never a fact. This was decided while nothing was built, which is the only time it is
cheap to decide.

**Two logs, one ordering.** `agents.events` is the harness transcript and keeps its closed twelve-verb
vocabulary, enforced at the type level. `log.entries` carries Locus's own domain events — `task.moved`,
`workflow.iteration_recorded`, `mail.sent`, `dispatch.enqueued` — with an open, per-kind versioned
payload. Putting `task.moved` in the telemetry enum would destroy the property that makes harness
output testable, so they stay apart and share one `stream_pos` counter per project instead. *"Everything
since N"* spans both without merging two orderings by timestamp.

**The fold is synchronous, in the append's own transaction**, so a projection is never stale and there
is no caught-up question. That is available only because the core is the sole writer — the same
property that makes a Postgres sequence the wrong source for `stream_pos`, since a sequence is assigned
at insert and made visible at commit, and two concurrent runs can commit out of that order.

**Telemetry is exempt from projection.** `agents.events` is appended raw; runs-by-hour, cost per
session and verify rates are queries against it. It is the hot path — written on every tool call of
every run — and fold work inside that transaction would tax the highest-volume writer in the system to
serve dashboards that tolerate a query.

**The line the carve-outs sit on is not a schema list:**

> The fold produces everything except what a model or a clock produced.

| Carve-out | Where | Why it cannot fold |
| --- | --- | --- |
| **Embeddings** | `memory.store`, `wiki.pages` | a model output, not a function of the events behind its text, and not reproducible across embedding-model versions |
| **Decay and confidence** | `memory.store` | a function of wall-clock time; folding it means re-deriving on every read, or writing tick entries so the log can model a clock |

Both are **declared, not discovered** — a `carve_out` annotation, and a schema test that fails when a
new non-foldable column appears without one. The facts and pages themselves still fold; only these
columns sit outside. **`locus rebuild` does not touch them**, because they were never derived from the
log and nothing exists to re-derive them from.

**What this costs, stated once.** Every `(kind, v)` ever written must stay foldable forever, and a fold
meeting an unknown one **halts naming the offending `stream_pos` rather than skipping** — a skipped
entry yields a projection that is quietly wrong, which is the failure event sourcing exists to prevent.
Locus already carries permanent schema-evolution cost against eleven third-party harness formats it does
not control; this adds a second obligation for events it does own. Owning them is the difference.

**And it does not reduce the backup requirement — it sharpens it.** The log lives only in the Postgres
volume, so losing the volume loses the thing everything else rebuilds *from*; and the two carve-outs
cannot be replayed at any price. A restore brings back the log and the carve-outs, and `locus rebuild`
regenerates everything between.

### What a session is

The word is overloaded — ACP has sessions, every harness has sessions, and the UI has panes. Locus
needs one definition, and this is it:

```
Project
└── Session          a durable, named thread of work with ONE agent
    ├── Run          one container lifetime = one ACP conversation
    │    └── Turn    one prompt → one response
    ├── Run          (after a loop reset: new container, same session)
    └── Run
```

| | **Session** | **Run** |
| --- | --- | --- |
| Bounded by | you closing it | the container exiting |
| Holds | agent@version, its branch on the local remote, the board task it serves, core-memory base, pane state | events, token usage, exit status, artifacts, **the resolved model id** |
| Resumable | yes — by starting another run | no; a run is over when it is over |
| Maps to | the harness's own conversation id, where it has one | one ACP process and container |
| Cost | the sum of its runs | measured directly |

**The session is what survives the reset.** That is the whole reason for the split. The Ralph-loop
pattern the field converged on — pick, act, validate, commit, *reset the context* — needs something
that persists across resets and something that does not. The run is the thing that resets; the session
is the thing that accumulates. A workflow iteration ends a run and starts a new one **in the same
session**, and memory, branch, and task linkage carry across because they belong to the session.

Three consequences worth stating:

- **A Locus session is not the harness's session.** The harness's own session maps to a *run*, and
  resume belongs to Locus rather than the harness: the next run is primed from the session's own
  events. Where a harness has a native session id the core stores it on the run and hands it back,
  which is an optimization, not the mechanism.
- **A terminal you drive yourself is not an agent session.** It is a separate human-work pane with no
  agent events or cost attribution.
- **Chat is a session**, with the designated spec agent. Nothing special about it.

### Handoffs — a payload for a mechanism that already exists

The guardrails already **kill and reassign after three stuck iterations**, and a session already
belongs to exactly one agent. Put together, those mean work changes hands regularly and currently
arrives with nothing: the successor inherits a branch and a task, and rediscovers everything else.

A **handoff** is that missing payload. `locus handoff <agent> --why …` ends the current session and
opens a new one on the same task and the same branch, linked by `handed_off_from`, carrying one
structured artifact:

```
handoff
  goal              what this work is for, restated
  done[]            what is finished, each with the evidence
  remaining[]       what is not, in the order it should be taken
  attempted[]       what was tried and did not work — the expensive half
  decisions[]       choices already made, so they are not re-litigated
  open[]            questions the successor inherits
  branch · task · artifacts[]
```

**The successor reads the handoff, never the predecessor's transcript.** That is the whole point: a
transcript is long, mostly irrelevant, and replaying it hands over the confusion along with the
context. `attempted[]` is the part that pays for the mechanism — without it the next agent's first act
is to retry what just failed.

Four things trigger one, and they are the same mechanism each time:

| Trigger | Who decides |
| --- | --- |
| Stuck — three iterations with no progress | the guardrail |
| Context exhausted | the run supervisor |
| The work needs a different role — builder to reviewer, or to a specialist | the workflow graph |
| You reassign it | you |

A handoff is **not** mail and **not** `locus agent invoke`. Mail is a message between agents that both
keep working; invoke is a nested run that returns to its caller. A handoff transfers ownership and does
not come back.

### Artifacts — what you review instead of tool calls

An **artifact** is a structured deliverable an agent produces to communicate what it did and what it
intends. Antigravity's framing is the right one: *"You do not need to carefully monitor every
individual tool call or step synchronously; instead, you review high-level deliverables at key
milestones."* With N agents running, reading transcripts does not scale. Reading deliverables does.

| Kind | Produced when | Lands from |
| --- | --- | --- |
| `plan` | before the agent starts changing things | the agent, as markdown |
| `diff` | on each meaningful change | the run's branch |
| `diagram` | when structure is worth a picture | the agent |
| `image` | a screenshot of a page or element | `locus browse screenshot` |
| `recording` | a playback of the agent's UI actions | `locus browse` |
| `walkthrough` | **on completion** | generated from the session |

**The walkthrough is the one that earns its place.** When a session finishes, it produces a concise
summary of what changed — with the screenshots and recordings inline — so you can catch up on work you
did not watch. That is the actual answer to "six agents ran overnight, now what."

**Artifacts are commentable, and a comment steers.** You leave inline feedback on a plan, a diff, or a
screenshot; the comment routes back into the session that produced it and the agent responds — the
run is still live while the task is unfinished, which is when comments actually arrive. A comment left
after a session's last run has exited is delivered by starting the next one. It is
the PR review interaction, applied to plans and images rather than only code — and it is the same
mechanism as the agent-authored PR flow in M7, so it is one implementation, not two.

**Artifacts are also where context goes to be forgotten.** A 60KB test log or a long research pass
does not belong in a context window, but it does need to be reachable — so the compaction hook writes
anything over a threshold as an artifact and leaves **a one-line summary and an id** in its place. The
agent fetches the body with `locus artifact get` if it turns out to matter. Same rule as memory, tool
docs, and images; this is the fourth surface it applies to, and the one that catches everything the
other three do not.

That makes artifacts do two jobs, so the kinds split by **whether a human is meant to see them**:

| | Kinds | Appears in the inbox |
| --- | --- | --- |
| **Review** | `plan` · `diff` · `diagram` · `image` · `recording` · `walkthrough` | yes, when it needs you |
| **Reference** | `finding` (an agent's own research or summary) · `payload` (an offloaded tool result) | **never** — they are storage with a handle |

Without that split the inbox fills with an agent's own scratch, and the one surface built to protect
your attention becomes the one that spends it.

**Text artifacts are rows; media is a file the row points at.** A plan, a diff, and a walkthrough are
text and live in Postgres, which compresses them at rest already. A screenshot or a recording is
megabytes, so the bytes land under `/var/lib/locus/artifacts/<project>/<run>/` and the row carries the
path, the media type, and a `sha256`. Backup covers both trees or it covers neither — that is the
reason to decide it here rather than at the first 40MB recording.

#### Two representations: one you look at, one the agent reads

Media is stored once for the human and derived on demand for a model, because the two want opposite
things. The rules come straight from `local-dx`'s `image-processing` skill, which exists for exactly
this problem:

| | Stored | Agent-facing |
| --- | --- | --- |
| `image` | WebP q80, longest edge capped at 2560 | **OCR text if the shot carries text**; otherwise downscaled to 1500px |
| `recording` | WebM as the browser produced it, capped by duration | extracted keyframes, never the clip |
| `diff` · `plan` · `walkthrough` | text in Postgres | the same text |

**Text is cheaper, searchable, and quotable; pixels are none of those.** An error dialog, a terminal
capture, or a failing assertion is text wearing a screenshot's clothes, so `locus artifact get
--for-context` OCRs it and returns characters. Only appearance — a layout, a rendering bug, a diagram
— justifies sending pixels, and 1500px on the longest edge carries all of it. Past that you pay tokens
for detail no model uses.

Three rules that keep this from going wrong:

- **The stored copy is the original.** Derived representations are cached beside it and regenerable;
  nothing overwrites the artifact a human will open.
- **OCR is lossy on tables and low-resolution text.** When it looks wrong, the agent gets the image
  instead — a bad transcription asserted as fact is worse than the pixels it saved.
- **Dimensions are metadata.** Deciding how to handle a shot never requires loading it.
- **Media has a retention policy; text does not.** Recordings and screenshots are pruned with their
  run after 30 days unless the run is linked to a PR or to a task in Done — the two cases where the
  evidence is the point. Text artifacts are small enough to keep forever, and the trace depends on
  them surviving.
- **Encoding is host-side and in Rust** — the `image` crate for resize and WebP, `ffmpeg` for
  keyframes, `tesseract` for OCR. The skill's `sips` is macOS-only and Locus ships on Linux too, so
  the tool list differs even though the rules do not.

This is also why the walkthrough is affordable. A session that produced forty screenshots inlines
forty OCR blocks and a handful of images, not forty megabytes.

Artifacts arrive in the inbox when they need you.

### The user inbox

With N agents running, your attention is the bottleneck. **Everything that needs you arrives in one
inbox** — not scattered across panes, badges, and tiles you have to go looking at.

The inbox is **you as a participant in the same mail system agents use**. `locus ask` from an agent,
a `Gate` waiting on approval, a guardrail trip, a contradiction found at wiki ingest, a workflow goal
awaiting sign-off, a finished run needing review — all of it is a message addressed to you, threaded,
with the session it came from.

**Silence is the default.** A session working normally produces nothing. The inbox tells you when
something *needs you*, not when something *happened*.

### Agents need real tools, not just a shell

Three capabilities that most of the field leaves agents to fake with `grep` and guesswork. Each is a
Rust client in `locus-cli` — one implementation, every harness — talking to a server that runs **where
the code is**, which is the agent's own container:

| Capability | CLI | Why it matters |
| --- | --- | --- |
| **Semantic navigation** | `locus lsp def\|refs\|hover\|symbols\|diagnostics\|rename` | An agent grepping for a symbol reads ten files to answer what one LSP call answers exactly. Cheaper in tokens *and* correct on overloads, shadowing, and re-exports |
| **Debugging** | `locus debug break\|run\|step\|stack\|eval\|vars` | An agent that can inspect a live stack stops guessing at runtime state from print statements |
| **Browser validation** | `locus browse open\|click\|fill\|assert\|screenshot` | An agent that changed a UI can look at it |

**On demand, not always on.** These are tools in an agent's allowlist, resolved from the marketplace
like any other. An agent that does not need a debugger does not get one, and pays nothing for it.

**Where each server runs:**

- **LSP and DAP run in the agent's container**, against that run's clone — the host's language servers
  index *your* working copy, which is a different tree. The Rust client is shared; the server is local
  to the code it is answering about. The host LSP supervisor continues to serve the editor panes
  separately. One implementation, two deployments.
- **The browser is a sibling container**, one per project (`locus-svc-<project>-browser`), driven over
  the project network. Headless Chromium in every agent image would be gratuitous; one per project on
  machinery `locus svc` already provides is not.

**This is what makes screenshots free.** The agent's app runs on `$LOCUS_PORT` in its container; the
browser container reaches it over the project network; the screenshot lands as a run artifact on the
board card. Cursor 3 ships this as "cloud agents generate demos and screenshots of their work so you
don't need to pull code to review" — here it falls out of capabilities we needed anyway.

**And it settles the DAP question.** An earlier draft accepted "no debugger" as CodeMirror's cost.
That was wrong once agents need to debug: the core needs a **DAP client** regardless.

**The debugger is for agents. There is no debug UI, and that is a decision.** No breakpoint gutter, no
variables pane, no step buttons — the entire surface is `locus debug` inside a container. You debug in
your own editor, on your own checkout, with the tools you already have; Locus does not compete for
that job. What you see of an agent's debugging is what it reports: the stack it captured, the value it
found, the artifact it filed.

Two things follow. The client is a fraction of the cost of an extension host, because a UI is most of
what makes a debugger expensive. And the design does not carry a half-built pane waiting for someone
to want it — if that changes, it changes as a new decision against a client that already works.

#### Debugging — the session is the core's, the CLI is stateless

A breakpoint set by one command has to still exist when the next one runs, and each `locus debug`
invocation is a separate process that exits. So **the debug session lives in the core, not in the
CLI**: `locus debug` opens or attaches to a session keyed by run id, the adapter process is long-lived
inside the agent's container, and every command is a request against it. The CLI holds nothing.

```
locus debug start [--config NAME]     launch under the adapter; --config names a project run config
locus debug break FILE:LINE [--if EXPR] [--log FMT]
locus debug run|step|next|finish|continue
locus debug stack|vars [--frame N]|eval EXPR
locus debug stop
```

Five things this has to get right, each of which is a way agents lose time:

- **Logpoints before breakpoints.** `--log` prints a formatted message and keeps running; `break`
  stops the world. An agent that stops the world then has to remember to continue it, and a stopped
  process that nobody resumes looks exactly like a hung run. Logpoints are the default advice in the
  tool's own docs blob.
- **A paused program is not an idle agent.** The idle guardrail counts events on the run's stream; a
  debug session parked at a breakpoint suppresses it, because the agent is working and the program is
  not. Without this every real debugging session trips a guardrail at 60 seconds.
- **`--config` comes from project settings**, the same place the run script lives. Debugging is not a
  different way to start the app; it is the same command under an adapter.
- **Adapters are tools.** `codelldb`, `debugpy`, `js-debug` are marketplace entries in the agent's
  allowlist, so they are in the image or they are not available — same rule as every other tool, and
  the reason `locus debug` has honest coverage limits rather than pretending.
- **The adapter dies with the run.** No cleanup path, because the container takes it.

#### Browser testing — one container, one context per run

The browser container is shared by a project's agents, which makes isolation the whole problem: two
agents driving one page is two agents fighting. **Each run gets its own Playwright browser context** —
own cookies, own storage, own pages, cheap to create — inside the one shared browser. The container is
per project; the context is per run.

```
locus browse open URL              relative to the run's own app: http://<its container>:$LOCUS_PORT
locus browse click|fill|press SELECTOR [VALUE]
locus browse assert SELECTOR [--text S] [--visible] [--count N]
locus browse screenshot [SELECTOR]  → an image artifact
locus browse record start|stop      → a recording artifact
locus browse console|network        what the page logged, what it fetched
```

- **The app is started by the container, not by the agent.** If the project declares a run script it
  starts at container start, backgrounded, and `locus browse open` blocks until the readiness probe
  passes. An agent should not have to remember to start its own app, and an agent that forgets
  produces a screenshot of a connection error and reports it as a UI bug.
- **`assert` exits non-zero and prints structured JSON**, so a workflow's `Verify` node can use it
  directly. This is the difference between a browser an agent plays with and a browser that gates a
  merge.
- **Auto-waiting, not sleeps.** Playwright waits for actionability by default. An agent writing
  `sleep 2` is a flaky test being born, and the docs blob says so.
- **The browser gets no egress by default.** It exists to reach the agent's app on the project
  network. A test that genuinely needs a third-party origin is a project setting, named and audited —
  otherwise the browser is a clean way around the egress policy the whole sandbox model rests on.
- **`console` and `network` matter more than pixels.** A failing UI usually explains itself in a
  console error, which is text; a screenshot of it costs tokens to say less. Same rule as the OCR
  path: text first, pixels when appearance is the question.
- **Screenshots and recordings land as artifacts automatically**, with the derived agent-facing
  representation built on demand. The agent does not upload anything.

### The wiki — ingested and typed, not a blank page

Taking the shape from [`SamurAIGPT/llm-wiki-agent`](https://github.com/SamurAIGPT/llm-wiki-agent)
(MIT). Its premise is the right one: *"Most knowledge tools make you search your own notes. This one
reads everything you've collected and writes a structured wiki that compounds over time."* A wiki
nobody writes is a wiki nobody reads.

**Pages have kinds.** Not one flat namespace:

| Kind | Holds | Created by |
| --- | --- | --- |
| `source` | one summary per ingested document — an ADR, a spec, a PR body, an external doc | ingest |
| `entity` | a person, service, repo, or system referenced across sources | auto, on first mention |
| `concept` | an idea, pattern, or convention this project uses | auto, on first mention |
| `synthesis` | an answer to a question, filed back as a page so it is asked once | `locus wiki query` |
| `decision` | why something is the way it is — the ADR's home in the wiki | ingest or human |
| `overview` | a living synthesis, revised on every ingest | ingest |

**Ingest, not authoring.** `locus wiki ingest <path|url>` reads a document, extracts entities and
concepts, writes or updates the pages, and links them. `markitdown` handles PDF, DOCX, PPTX, XLSX,
HTML and the rest, and is already installed on this machine. The GUI editor still exists — a human can
always fix a page — but the default path is that the wiki is *derived* and then curated, not composed
from nothing.

**How a contradiction is found.** The new statement's embedding retrieves its *k* nearest existing
assertions, and only those go to a model to adjudicate — agree, contradict, or unrelated. Bounded by
construction: ingest cost scales with what the document says, not with how much the wiki already
holds. A contradiction verdict carries both statements and both sources, because a flag you cannot
adjudicate yourself is just an alarm.

**Contradiction flags at ingest time, not query time.** This is the idea most worth stealing. When a
new source contradicts an existing statement, the conflict is raised *when it lands* — as a row in
`wiki.contradictions` and a card on the board — rather than discovered months later by whoever
happened to read both pages. The same detection serves memory: a store-tier fact that conflicts with a
wiki statement is the same problem.

**A wiki linter.** `locus wiki lint` reports orphan pages, broken links, entities mentioned but
never given a page, and assertions with no source. This is the same discipline `dx-lint`
already applies to code, pointed at knowledge.

**A graph view, nearly free.** Pages are nodes, `[[wikilinks]]` are edges. `solid-flow` is already in
the app for the Workflow Canvas, so rendering this costs a palette, not a subsystem.

### Knowledge, as one model

Four kinds of knowledge, and three of them are **not** memory. This is the scoping decision that keeps
the store small enough to be trustworthy:

| Knowledge | Mechanism | Why not the memory store |
| --- | --- | --- |
| Code structure — signatures, call graphs, dependencies | `codanna`, queried live | AST-derived, always current, rebuildable, no LLM cost |
| Subsystem specs — how a part works, and why | wiki, one page per subsystem | curated; consolidation cannot corrupt what a human wrote |
| Project constitution — objectives, stack, conventions | small, always loaded, **human-written** | LLM-written context files measure ~3% worse at 20%+ more cost |
| **Agent observations** — what failed, what was tried, preferences | **the memory store** | genuinely accumulated; nothing else can hold it |

`codanna` is already installed and already does the code graph — `index`, `retrieve` over symbols and
relationships, `parse` to AST JSONL, as a CLI. Embedding source text captures surface similarity and
misses structure: knowing that `process_payment` calls `validate_card`, which depends on
`CardProvider`, needs a graph, not a nearest neighbour.

Codified Context reached the same split empirically across 283 sessions, and states the reason
plainly: single-file manifests **do not scale beyond modest codebases** — a 1,000-line prototype fits
in one prompt, a 100,000-line system does not.

### Memory

Four layers, split by **lifetime**. Locus owns three; the one it does not own is the one every
memory product tries to build twice.

| Layer | Lives in | Dies when | Owner | Shared |
| --- | --- | --- | --- | --- |
| **Working** | the context window | the run ends | **sub-harness** | no |
| **Short-term** | probation buffer | promoted, or aged out | Locus | no |
| **Long-term** | Postgres | decays below threshold | Locus | yes, by role |
| **Written** | **git** — artifacts | never | the repo | everyone |

Written memory is **artifacts**: wiki pages, specs, ADRs, plans, walkthroughs. Agent-authored,
permanent, committed, and needing no human gate — MetaGPT structures multi-agent shared memory as
exactly this. The **constitution** is separate and is *not* a memory layer: human-authored
instructions, where the ETH Zurich finding applies and nowhere else.

**There is no shared short-term.** Two agents sharing scratch state is Hermes's documented failure —
*"two writers sharing one home will compound each other's entries into state neither of them (nor
you) authored."* Sharing begins at long-term, after consolidation, so the promotion boundary and the
trust boundary are the same line.

**Probation is project-scoped, not session-scoped.** Cluster density is cross-session by nature: one
session yields one observation, and the pattern only appears once three sessions have yielded three.
What dies at session close is the session *context* — thread, branch pointer, pane.

#### Capture — hooks, not tools

`locus-hook`, one binary, materialized into every container and registered through each harness's own
hook config. JSON on stdin, plain text or JSON on stdout, **exit 0 on every failure path**.

Tools are agent-initiated, so the model must *remember to remember* and consistency is not
guaranteed. Hooks are passive and deterministic, and cost nothing in context when they emit nothing.

**Hooks log and inject. They never think.** A hook cannot reuse the agent's LLM, and it fires on every
tool call — so any model call inside one taxes the whole run. Two rules follow: the injection path
carries a **100ms timeout** and emits nothing on expiry; and the logging path **never touches the
socket synchronously** — it appends to a local buffer and returns, with a background flush. That also
makes the *"SessionStart fires before servers finish connecting"* gotcha a non-event.

#### Injection — a catalog, not content

`SessionStart` emits **paths and one-line summaries**, capped at **800 tokens ≈ 40 entries**. Bodies
arrive through `locus memory recall` when an agent asks. This is just-in-time retrieval, and it also
dissolves the routing problem: bodies never pre-loaded means over-retrieval cannot occur.

The cap is derived, not chosen. The maximum *effective* context window is far below the marketed one —
11 of 12 frontier models drop below 50% of baseline at 32K tokens — and static anchors should hold
under 10–15% of it. That leaves 3–5K tokens for system prompt, tools, and memory combined. Forty
entries is comfortably above what real projects occupy: Codified Context's production system
consolidated to 34 subsystem documents; one production memory store went from 444 memories to 16
summaries.

Exceeding 40 entries is a **consolidation trigger**, not an eviction.

Output must not begin with `{`, which Claude Code parses as JSON rather than context.

#### Promotion — three checks

1. **Re-verification.** Locus can genuinely verify a subset, which most memory systems cannot: a fact
   about a signature against `codanna retrieve`, a flaky test by running it, an approach against that
   run's own verify result. Preferences fall through to provenance instead.
2. **Deduplication.** Path addressing does it — same path means rewrite, bump version, set
   `supersedes`. Subject-aware embedding runs first so entity confusion cannot merge unrelated facts.
3. **Importance.** Not predicted — **measured.** Locus logs what was injected, what was recalled, and
   whether the run passed verify. A memory recalled into a passing run is important; one never
   recalled is not.

Promotion is **event-driven on cluster density**, not on a timer: candidates wait until **three**
target the same path. Two are coincidence; three are a pattern. Originals are archived, never deleted.
On buffer overflow, promote by score and drop the rest **with a log line** — never silently.

#### Decay — two drivers, selected by content

Memories naming a **code symbol** invalidate on *change*, not time. Git blame gives the signal;
`codanna` makes it precise:

| Signal | Action |
| --- | --- |
| Signature changed | invalidate — actively misleading, not merely old |
| Body changed, signature stable | flag for re-verification |
| AST unchanged | no change — it was a formatting or rename commit |

Everything else — preferences, strategies, process observations — ages on an Ebbinghaus curve, where
`active_days` counts only days the project saw a run:

```
effective_λ = base_λ × (1 − importance × 0.8)
strength    = clamp(importance × e^(−effective_λ × active_days)
                    × (1 + recall_count × 0.2), 0, 1)
prune below 0.05
```

Half-lives by category: `strategy` 38d · `fact` 24d · `assumption` 19d · `failure` 11d. Unpromoted
candidates age on the shortest curve, which bounds buffer growth without a separate mechanism.

**Chain-aware pruning** solves the problem that would otherwise sink this. A rare critical
instruction — *"never call the production database directly"* — is exactly the low-frequency,
high-importance detail that vanishes by the third compression pass, because frequency and importance
are anticorrelated for the memories that matter most. The answer is not an exemption list: **a decayed
memory survives if any graph neighbour is still strong.**

**Cold-start guard:** a memory is not eligible for pruning until it has survived one keeper pass in
which a query could plausibly have matched it. You cannot conclude a memory is useless if it was never
given the chance to be useful — and without this, a new store prunes its own seed corpus.

#### Recall and routing

Two rounds — role and visibility filter, then BM25 + vector hybrid, then graph expansion — ranked by
**similarity × strength**, so decay both prunes and demotes.

Retrieval depth is the lever, and it **reverses by task class**. On factual QA accuracy rises
monotonically with *k*. On agentic tasks the sign flips: success falls from 32.1% at k=1 to ~25% at
k=5, and flat retrievers drop *below* the 22.4% no-memory baseline — over-retrieved agents do not
simply fail, they wander.

| Task class | Substrate | k | Graph expansion |
| --- | --- | --- | --- |
| Code generation / edit | flat, similarity only | 1 | off |
| Planning / strategy | distilled procedural cues | 1 | off |
| Research / review | structural, hierarchical | high | on |

`task_class: code | plan | research` sits on the agent definition, defaulting to `code`; a workflow's
`Agent` node overrides per role. Turn-level body injection is **off** for `code` and `plan` — a missed
injection costs one tool call, while a wrong one measured below the no-memory baseline.

Heavy machinery is not free: on code generation, graph and hierarchical substrates incurred 3.7–8.0×
overhead while landing *strictly below* having no memory at all. Causal-parent traversal survives
because it fires on demand during debugging rather than expanding every query.

#### The keeper

An ordinary agent definition — Markdown, versioned, nothing special — running at `high` tier because
it is not latency-bound, triggered on **genuine idle** — the same signal the guardrail uses, raised to
the project: no run on this project has appended an event for the idle window. Locus can detect it
because every run's events land in one place. The primary agent has **no memory-edit tools at all**; memory management is the keeper's
job exclusively.

Per pass: read events since the watermark → cluster candidates → run the three checks → merge into
notes at their paths → recompute strength → prune with chain-awareness → archive originals.

#### Seeding

An empty store degrades cleanly: no catalog, no injection, no recall — agents run exactly as they
would with no memory layer, which is the correct baseline. But it need not stay empty. Every project
here already has written memory in git — ADRs, specs, READMEs, existing `AGENTS.md` files. `locus wiki
ingest` reads those on day one, so the first keeper pass has a corpus rather than a blank store.

### Token discipline

If token usage really explains most of the variance in how a run goes, then tokens are not an
operational concern that gets attention when a bill arrives — they are a design constraint on every
surface that puts bytes in front of a model. Four levers, in the order they pay.

#### 1. Prefix stability, because cache rate is the number

Cached reads bill at a fraction of fresh input, so **a long session with a stable prefix is cheaper
than a short one that busts the cache every turn**. What breaks it is content that changes near the
*front* of the prompt: a rotating timestamp, a re-ordered tool list, an instruction file edited
mid-session. Locus writes that entire prefix — materialized config, base-context, the memory catalog —
which makes cache rate something the design controls rather than observes.

Five rules follow, and all of them are cheap if applied from the first commit:

- **Materialization is deterministic.** Sorted file order, sorted tool lists, no timestamps, no run id,
  no hostname. The same agent with the same tools produces a byte-identical config tree, so two runs
  share a prefix instead of each paying to build one.
- **The config tree is frozen for the life of the run.** It is already discarded at exit; nothing may
  rewrite it mid-run. Editing a skill affects the *next* run.
- **The memory catalog is a snapshot, not a live view.** Taken at `SessionStart`, unchanged until the
  run ends — Hermes's frozen-snapshot mechanic, adopted for this reason and not only for memory
  hygiene.
- **Mutable content goes last.** Anything per-run — the canary token, the run's port, its branch name —
  sits at the end of the assembled context, after everything shared. A per-run value near the front
  costs every other run's cache.
- **Turn-level injection stays off for `code` and `plan`.** Already the rule for retrieval quality;
  it is also the single most reliable way to mutate a prefix mid-session.

`usage.cache_read` against `usage.input` is already on every event, so **cache rate is a column, not a
project.** Below ~80% on a long session means something in front is moving, and the run that did it is
identifiable.

#### 2. Prevention at the tool boundary

The cheapest token is the one that never enters context. `local-dx` puts this in front of the tool
rather than behind it: `rtk` rewrites verbose commands and compacts their output *before* the result
is appended, reported at 60–90% savings on ordinary development operations. Locus already materializes
hooks into every container, so **the same interception ships as a `PreToolUse` hook in the base image**
— one implementation, every harness, and it needs no cooperation from the agent.

The rule it enforces: a tool high on bytes but low on calls is returning too much per call. Narrow the
read, add a line range, compact the output.

#### 3. Diagnosis, as a query

`token-optimizer top` ranks tools by how much result payload they put into history, because *halving
the biggest contributor beats eliminating three small ones*. Every `tool_result` is already a
normalized row here, so that ranking is a `GROUP BY` — per agent, per project, per harness — rather
than a tool to build. The dashboard reports it beside spend, and it is what makes "this agent is
expensive" actionable instead of merely true.

#### 4. Summaries with handles, never bodies

This is one rule the design already applies three times, and it is worth naming so the fourth case
does not get decided differently:

| Surface | What the agent gets | Body on demand via |
| --- | --- | --- |
| memory | 800-token catalog of paths and one-liners | `locus memory recall` |
| tool docs | a one-line catalog, ~15 tokens per tool | `locus tools docs <name>` |
| tool results over threshold | a summary line and an artifact id | `locus artifact get` |
| artifacts | OCR text or a downscaled frame | `locus artifact get` |
| code structure | `codanna` symbol and relationship queries | reading the file |

**Nothing is pre-loaded that can be fetched.** Retrieval-augmented context beats whole-file loading for
exactly this reason, and it also dissolves the over-retrieval problem: a body that was never injected
cannot crowd out the work.

One asymmetry worth stating because it aims the effort: **output tokens cost roughly five times
input**. That is what the `output-styles` extension is actually for — an agent that answers in three
sentences instead of thirty is a cost decision, not a style preference — and why every `locus` command
emits compact JSON rather than pretty-printed, packing uniform tables where the row count justifies it
(50–60% smaller than minified JSON on tabular data).

### The board

Deliberately small. Fixed columns across every project — not configurable — and only the gating that
something else already depends on.

```
Ready → In Progress → Testing → Reviewing → Waiting For Approval → Done
```

Four of the six map onto machinery already in the plan: **In Progress** is a run, **Testing** is the
task's verify command, **Reviewing** is a reviewer agent or a `Gate` node, and **Waiting For Approval**
is a human decision — which means it is an inbox item, not a place to go looking.

**Blocked is a status, not a column.** It shows as an icon and is orthogonal to progress, because a
task can be blocked at any point on the line, not only before it starts. Dependency auto-unblock
clears the *status* when a predecessor completes; it never moves the card.

```
task
  summary · description
  column                Ready | In Progress | Testing | Reviewing | Waiting For Approval | Done
  blocked               status, with the reason and what would clear it
  assigned_agent        nullable — unassigned is normal
  project · repo        which of the project's repos
  session               nullable — the session working it
  blocked_by[]          generated from the workflow graph, not hand-drawn
  verify                the runnable check
  evidence[]            run + the events that justify a transition
  github_issue          nullable, linked by explicit action in either direction
```

**Evidence proves the requirement was met, not that the feature is right.** Sengupta et al. report a
payments backend that passed CI in two cycles against an adversarially-written suite and still shipped
two business-logic failures, because neither behavior was in the contract. *"The harness built what
the contract specified; the contract did not fully capture the intended behavior."* No amount of
verification reaches outside its requirement, which is why **contract completeness is the highest-
leverage unsolved problem in any system shaped like this one** — and why the planning module's
elicitation, not its audit, is where the quality actually comes from.

**Two gating rules, and no more:**

- An agent cannot move a card to **Done** without evidence.
- **Blocked** clears automatically and never manually — it is derived from `blocked_by`, so clearing it
  by hand would just be lying about a dependency.

Everything else is unrestricted. You can drag anything anywhere; the constraints exist to stop an
agent asserting completion, not to stop you working.

### The planning module

A guided conversation that produces a reviewable plan. It runs over **ACP**, because it is a
conversation rather than a terminal, and nothing it produces reaches the board until you approve it.

**Three agents, not one.** The split is structural, not stylistic: measured on ProgramBench, single-pass
agents *"terminate early by agent choice, consuming a median of only 22–177 turns"* — exploration and
implementation compete for the same attention and implementation wins. Splitting them lifted test pass
rates 6.9–21.3% relative. Separately, ClarifyCodeBench found **capability decoupling**: strong code
generation does not translate to effective clarification, and more reasoning effort yields only
marginal gains at spotting ambiguity.

| Agent | `task_class` | Job |
| --- | --- | --- |
| **Interviewer** | `plan` | Drives the questions, holds the state, writes the artifacts |
| **Researcher** | `research` | Facts, prior art, feasibility. Never asked for intent |
| **Auditor** | `plan` | Fresh context, adversarial. Grades the spec it did not write |

That the three roles want the three different retrieval regimes — `research` broad, `plan` and `code`
narrow — is a useful cross-check on the `task_class` enum, which was derived for memory and not for
this.

#### The sequence

```
1  INPUTS       goal · project · target repo · involved repos
                optional: extra tools, workflow override
                     ↓
2  ORIENT       researcher, bounded and once
                  index target + involved repos · pull relevant wiki and decisions
                  prior art for the goal · resolve the marketplace index
                     ↓
3  CONVERSE     interviewer drives the question loop
                  ├─ ask you              intent, priorities, trade-offs
                  ├─ dispatch researcher  fact, feasibility, prior art  ──┐
                  ├─ scope decision → you inline, not a separate gate     │
                  └─ drop                 answered or irrelevant          │
                ←────────────────────────────────────────────────────────┘
                     ↓  nothing relevant left unanswered
4  SYNTHESISE   pass 1  completeness — make the implicit explicit: types, state
                        transitions, edge cases, trust boundaries, error conditions
                pass 2  reduction — cut what is unsupported, rewrite what is
                        ambiguous, so nothing downstream reads as mandatory by accident
                → spec · tasks · tool list · proposed workflow
                     ↓
5  AUDIT        auditor, fresh context: ISO/IEC/IEEE 29148 + the two-reader test
                  ├─ finding is a missed question → back to 3, ONCE
                  └─ residual → recommendation.open[] and confidence
                     ↓
6  RECOMMEND    the recommendation object, shown
                     ↓
7  OVERRIDE     you change anything
                     ↓
8  APPROVE  →   tasks land on the board
   REJECT   →   the draft stays here
```

**Synthesis is two passes, and the second one subtracts.** A completeness pass alone over-specifies,
and the failure that follows is specific: **downstream agents treat an unsupported requirement as
mandatory**, so a speculative clause becomes work someone does. The reduction pass exists to delete
it before it is load-bearing. This is the concrete mechanism behind the Specification Overfitting
guard — a rule that research cannot unilaterally widen scope stops the spec growing, and a pass whose
only job is to cut stops it staying grown.

**Orientation is separate from on-demand research** because inferring what you need and checking which
tools exist both require knowing what is already there — and doing that per question would repeat the
same lookups a dozen times. It is bounded because unbounded research before the questions are known is
where scope quietly triples.

**The audit loops back at most once.** A finding that says "this is ambiguous" is really saying "a
question was missed", which belongs back in the loop rather than in a report. But clarification quality
*degrades* as ambiguity density rises, so a third pass rarely helps where the second did not. Whatever
survives becomes a named weakness in the recommendation, not a blocker.

#### Human inputs

| Input | | Note |
| --- | --- | --- |
| `goal` | required | The frame. Set first, not derived |
| `project` | required | |
| target repo | required | Where the branch is cut and the PR lands |
| involved repos | optional, N | Read-only context. Cloned to `/context/<repo>` beside `/workspace`, indexed by `codanna`, and never pushed — the run's branch exists only on the target repo's remote |
| tools | optional | Added to whatever research finds |
| workflow | optional | Overrides the proposal |

**Write scope and read scope are different.** Only the target repo receives commits; involved repos
inform. A capability needed against an involved repo is a read capability, which is a different risk
from a write one.

**The goal is an input, not an output**, and that is what makes the question loop work: topics are
ranked by how much they bear on the goal, dropped when they do not, and the interview ends when
nothing unresolved still does. Without a stated goal every topic is equally relevant and the ranking
is arbitrary. The goal then does three jobs — it frames elicitation, it is what you approve, and it is
what `verify` checks against.

#### Scope changes are approvals, never automatic

Research can widen scope (a dependency nobody mentioned) or narrow it (prior art shows it is already
solved). **Both require you.** Narrowing is the more dangerous direction: an increase is visible in a
growing spec, while a silent reduction ships something missing and leaves no trace of which step
removed it.

They are a different interaction from a question, and are counted separately — a well-run interview
that surfaces three legitimate scope decisions should not score as inefficient questioning. A rejected
increase is recorded as a `decision`, so the next research pass does not propose it again.

This is also the concrete guard against Specification Overfitting: a researcher that cannot
unilaterally widen scope cannot inflate the spec.

#### Latent requirements

After the interview, a LENS-style pass infers capabilities that were described but never requested,
reasoning over the transcript together with what the infrastructure already is. Reported hit rate is
**75% judged useful by domain experts** — which also means a quarter are noise, so these are always
*proposed*, never adopted, each carrying the excerpt that implied it.

#### The audit rubric

ISO/IEC/IEEE 29148 turns "is this spec any good" into pass/fail questions: necessary,
implementation-independent, unambiguous, consistent, complete, singular, feasible, traceable,
verifiable, understandable. The set as a whole is graded on completeness, consistency, modifiability,
traceability. Acceptance criteria are the standard's *verifiable* characteristic, which is the bridge
to a task's verify command.

**One criterion is mechanised rather than judged.** *Unambiguous* asks whether two readers would reach
the same understanding — so give the requirement to two agents with no shared context, ask each to
restate what it requires, and diff the restatements. **Divergence is the ambiguity, and it names
itself.** This matters because a model faced with ambiguity silently picks one reading and builds it,
and this makes the fork visible before code exists.

The auditor also checks that the recommendation's restatement of the goal still means what you wrote —
the highest-leverage single check in the sequence, since everything downstream inherits that reading.

#### Outputs

| Output | Committed on approval |
| --- | --- |
| **spec** | wiki page, contract separated from design |
| **tasks** | board — vertical slices, hardest first, dependency edges drawn, verify command each |
| **tools** | required capabilities resolved against the index, gaps flagged |
| **workflow** | **proposed, not drawn** |
| **recommendation** | the object you approve |

The workflow is proposed rather than committed because a graph you did not lay out is one you will not
trust, and the goal it carries is your approval gate.

Vertical slices ordered **hardest first** — Walking Skeleton — so architectural risk surfaces before
later slices anchor on assumptions the hardest one invalidates.

```
recommendation
  goal            restated as understood   ← drift check against what you set
  approach        the recommended one
  alternatives[]  considered, each with why-not
  findings[]      research results, each → source
  risks[]         severity + trigger
  scope           in / out, explicit
  confidence      + what would raise or lower it
  open[]          unresolved, and whether it blocks
  → spec · tasks · tools · proposed workflow
```

`alternatives[]` is what stops a rejected approach being re-proposed every planning pass, and
`confidence` is a named condition rather than a number — *"medium; high once the migration path is
confirmed"* is an action, a percentage is not.

#### Traceability, both directions

```
excerpt → requirement → task → run → evidence → PR
   ◀──────── why does this exist? and why doesn't it? ────────
```

Excerpts follow the W3C Web Annotation model: a quote selector (`exact` with `prefix`/`suffix`) beside
a position selector, because quote-anchoring survives re-rendering and offsets are precise, and each
repairs the other. An `exact` that matches more than once is **flagged for audit rather than silently
anchored to the first hit**. Since the transcript is immutable append-only events, an excerpt is
`(event_id, start, end)` plus the quote text, kept so the citation reads without a lookup.

The backward trace earns its keep on deletions: a requirement that was **removed** leaves no artifact
at all unless the decision is recorded.

#### Re-planning keeps spec and tasks in sync

Amend, never supersede — the same rule the memory store uses, where a path collision rewrites and bumps
`supersedes`. Tasks cannot all be rewritten, so:

| Task state when its requirement changes | Rule |
| --- | --- |
| Not started | rewrite in place |
| **In progress** | flag it and notify the session — never silently mutate a task an agent is working from |
| **Done** | never touch. Emit a *new* task for the delta, linked to the original |
| Requirement deleted | close as `superseded`, never delete — the trace has to survive |

#### Effort scales by a ratchet, not a question

Blast radius cannot be assessed up front, because determining it is what planning does — and asking you
to estimate it asks for a number you do not have. So planning **starts minimal and escalates on
evidence, never de-escalating**: more than one repo involved, a scope decision raised, research finding
no prior art, an answer contradicting an earlier one, or unresolved topics outnumbering resolved ones.

A one-line fix trips none of them and stays three questions. A new subsystem trips several in the first
few turns. This is the resolution of a real tension in the literature: dependence on good specs *rises*
with model capability, yet one team rebuilt a feature with **3.5+ hours and 2,577 lines of spec against
23 minutes of iterative prompting, for comparable bug counts**. Elicitation pays; document volume does
not.

#### Specialization records — the checklist a domain earns

The topic list is general: actors, triggers, inputs and outputs, error paths, limits, persistence,
concurrency, permissions, observability, migration and rollback, failure modes, out-of-scope. It is
hand-written once, because those topics are properties of software rather than of a project.

What *is* per-project is the layer above it. A **specialization record** is a domain's accumulated
requirements — payments demands idempotency keys, explicit state transitions, and trust-boundary
checks; auth demands session invalidation and privilege boundaries — injected into synthesis when the
goal touches that domain. Records are written by the calibration path, not by hand: a spec gap the
arbiter classified in a payments task is exactly the evidence that the payments record was missing a
clause.

Two rules keep them from doing harm:

- **Applied only above a confidence threshold.** Below it the pass runs without the record, because a
  wrong domain assumption injected into a contract is worse than no assumption — it arrives wearing
  the authority of accumulated experience.
- **They are wiki `concept` pages, not a new store.** A specialization record is curated prose about
  how this project does a domain, which is what the wiki already is. No fourth knowledge tier.

This is what turns failures into process rather than into retries, and it is the loop the field keeps
finding: a recurring bug becomes a regression test, a recurring spec gap becomes a record clause, a
recurring ambiguity becomes a compiler rule.

#### Open

- **The "I don't know what I want yet" entry point.** A goal is required up front, so something has to
  turn a vague idea into a goal worth grilling against. Whether that is a mode of this module or a
  separate one is **undecided**.

### Lego blocks

| Block | Is | Key constraint |
| --- | --- | --- |
| **Agent** | **Markdown + a tool list** — frontmatter over a prose body | Immutable once a run references it; edits create a new version |
| **Workflow** | a **canvas graph** that loops agents toward a **goal**, with guardrails | **`verify` is NOT NULL.** The goal is the approval gate; verify is the runnable check |
| **Schedule** | cron expression → workflow | Executions recorded with their verify result |

**There is no `Job`.** An earlier draft had one; a Workflow subsumes it, and running a single agent
ad hoc needs no wrapper at all. Three concepts, not four.

`verify` stays NOT NULL because it is what lets a workflow run unattended and still be trusted, and
why the dashboard can show green/red rather than "finished". The **token budget is optional** — a
nullable ceiling, not a required one, because most workflows do not need a number attached to them.

### Agents are Markdown

An agent is a Markdown file with frontmatter. Not a graph, not a compile step:

```markdown
---
name: reviewer
description: Read-only critic; runs on task completion
harness: any            # or a specific one, when it matters
model_tier: high        # low | medium | high | xhigh — mapped to a model in Settings → Harnesses;
                        # a missing tier falls back UP, never down
task_class: research    # code | plan | research — sets retrieval depth; defaults to code
tools: [rg, gh, cargo]  # allowlist, resolved against the marketplace index
skills: [audit-code]
rules: [no-secrets]
memory:
  scope: project        # agent | project — never cross-project
  write: propose        # none | propose | direct
                        #   none    cannot write memory at all
                        #   propose writes land in the probation buffer for the keeper
                        #   direct  writes straight to the store — the keeper only
---

You review code you did not write. Find the defect, not the style...
```

**This is the single biggest simplification in the design.** Every harness already reads exactly this
shape — `name` + `description` frontmatter over prose is what Claude Code, pi, Codex, and opencode all
expect. So "materialize the agent into the layout this harness reads" becomes close to an identity
transform plus a rename, rather than a compiler. The `layout.agents` entry in the harness contract
does almost no work, which is the best outcome available.

Two consequences worth stating:

- **`tools` is enforced, not advisory.** It is the container's install set *and* the allowlist. A tool
  absent from the list is absent from the image, so the agent cannot reach for it.
- **Composition moves to workflows.** An agent no longer nests other agents structurally. It can still
  call `locus agent invoke` at runtime if that is in its tool list — bounded in the core at **depth 3
  and fan-out 4**, which a workflow may lower and never raise — but *authored* composition is what the
  Workflow Canvas is for. Depth 3 with fan-out 4 is at most 21 containers, which one machine survives;
  depth 4 is 85, which it does not.

Agent definitions live in Postgres like everything else, with import and export as `.md` so they can
be reviewed in a PR or copied between machines when that is what you want.

#### Permissions are declared, never prompted

The human is never asked mid-run, because a headless run that stops to ask has nobody to answer it.
What an agent may do is set in two places, and the second may only ever **narrow** the first:

| Set on | Declares | Rule |
| --- | --- | --- |
| the **agent definition** | `tools` — the install set *and* the allowlist | the baseline. Absent from the list means absent from the image |
| the workflow's **`Agent` node** | a `tools` subset, a `network` tier, and `write` scope for this role | **subtracts only.** A node can take the reviewer's write access away; it cannot grant a capability the definition did not have |

Narrowing-only matters because the definition is what was reviewed. If a graph could widen, every
workflow would be a place to re-grant privileges, and reading an agent would stop telling you what it
can do.

Enforcement is the container, not the harness: an unlisted tool is not installed, the network tier is
applied at the proxy, and the run's branch is the only writable thing in reach. The harness's own
permission gate is switched **off** at launch, by the `argv` its harness file declares — which is why
`permission_request` firing means a misconfiguration rather than a decision waiting for you.

### The Workflow Canvas

`solid-flow` (`/dsnchz/solid-flow`) — a SolidJS port of React Flow with custom nodes, typed node
props, and handles. Chosen over Rete.js, whose renderers target React, Vue, and Svelte but not Solid.

The canvas authors one artifact: a **Workflow**.

**Node vocabulary.** A workflow is **a loop toward a goal**, not a one-shot pipeline. The shape the field converged on
(Huntley and Carson's "Ralph loop") is: pick → act → validate → commit → reset, iterating with a fresh
context each pass so confusion does not accumulate. Locus has a real advantage there — the four
ad-hoc memory channels that pattern relies on (commit history, a progress log, a task file, a context
file) are one queryable, scoped store here.

| Node | Contributes |
| --- | --- |
| `Goal` | what the loop is *for*. **This is the approval gate** — a person approves the goal before the loop runs — and it is also the termination condition |
| `Agent` | an agent definition, pinned by version, **plus a `role`** — the definition is reusable, the role is per-workflow. Carries this role's permission narrowing: a `tools` subset, a `network` tier, `write` scope |
| `Task` | a unit of work, optionally sourced from the board |
| `Loop` | the iteration construct: what repeats, and what resets between passes |
| `Condition` | deterministic routing. **No model in the orchestration path** — the graph decides, not an LLM |
| `Gate` | a checkpoint: human, or another agent acting as reviewer |
| `Verify` | the runnable success criterion. Required |
| `Guardrails` | the limits below, attached to the workflow |

**Where `Verify` runs.** In a **fresh container from the agent's own image, on the run's branch** —
never in the agent's container. Two reasons, and both are the same reason: the agent's container may
already be gone, and its filesystem holds whatever the agent did outside git. A check that passes only
on the machine that made the change has verified nothing. The command comes from the task or the node,
the exit code is the result, and stdout plus stderr are captured as the evidence the board requires
for Done.

**What a `Condition` can say.** A small total expression language over facts the run already produced
— no code, no model, no I/O, and no way to hang the orchestrator:

```
verify.passed · verify.exit_code · iteration · elapsed · tokens.used
events.count(tool_error) · events.last(kind) · artifact.exists(kind)
task.status · mail.pending
```

with `== != < > <= >=`, `and or not`, and parentheses. It is deliberately not a scripting language:
every operand is a column, so a `Condition` is a `WHERE` clause against the run, evaluable in the core
in microseconds and reproducible from stored events. Anything it cannot express is a `Gate` — which is
to say, a decision that deserved a person or an agent rather than an operator.

**The Ralph loop is a preset, not the only shape.** *pick → act → validate → commit → reset* is the
pattern the field converged on, and it is expressible in these nodes already — a `Loop` whose reset
starts a fresh run in the same session, a `Verify` as the validate step, a `Goal` as the termination
condition. Because it is a shape people want often, it ships as a **template**: `locus ralph --goal
… --verify …` runs one without opening the canvas, and dropping a Ralph preset onto the canvas
expands into the ordinary nodes so it can be edited rather than configured.

Two honest notes on it. It is **token-hungry by construction** — a fresh context every pass is the
point, and it is what the budget guardrail exists for. And it is **only as good as its verify**: a
loop iterating against a weak check converges confidently on the wrong thing, which is why `verify` is
NOT NULL and why the arbiter's noise class matters more here than anywhere else.

**Workflow guardrails.** Each is borrowed from a measured failure in an existing tool. Defaults apply
to any agent run; a workflow may tighten or relax them, and they are what make leaving a loop
unattended defensible:

| Guardrail | Default | Why |
| --- | --- | --- |
| `max_iterations` | 8 | Agents loop endlessly retrying the same broken approach without a hard stop |
| Forced reflection before retry | on | "What failed? What specific change would fix it? Am I repeating myself?" — reported to substantially cut stuck agents |
| Kill-and-reassign after 3 stuck iterations | on | A fresh context beats a confused one |
| Waiting ≠ idle | — | A run carries a `waiting` state with a reason — `ask`, `mail`, `debug-paused`, `gate` — and idle counts only time outside it. One mechanism, four callers: `locus ask`, `locus mail wait`, a debug session at a breakpoint, and a `Gate`. Without it every deliberate block reads as a stall |
| Idle detection | 60s | **No event on the run's stream for 60s**, while not blocked on `locus ask` — waiting on you is an inbox item, not idleness. The tile gets an **idle icon** and a **toast fires once per idle stretch**, never repeatedly. Agent Orchestrator's measured flaw was agents idling 90+s with nobody told |
| Wall-clock ceiling | none | Optional; a loop that cannot finish overnight should stop, not run to morning |
| **Token budget** | **none — optional** | When set, auto-pause at 85% and notify rather than draining silently. Nested agents multiply spend, so a ceiling is worth having available even if rarely used |

**A budget is optional; the accounting is not.** Rotifer cites Anthropic finding that *"token usage by
itself explains 80% of the variance"* in agent task performance. If that holds even roughly, tokens
are not a cost line to watch — they are the strongest single predictor of whether a run went well, and
a run that passes verify on four times the tokens is a worse run wearing a green tick. So every run
carries usage whether or not a ceiling was set, agent trust is weighted by tokens per passing run, and
the dashboard reports both. The ceiling stays optional because most workflows do not need one; the
number is mandatory because without it the dashboard cannot tell a good run from an expensive one.

**Pause means the loop stops being fed, not that a process is frozen.** The supervisor lets the current
turn finish, holds before the next iteration, and notifies; the container stays up so its state is
still inspectable. `SIGSTOP` on a harness mid-request would leave sockets half-written and a model
call in flight, which is a worse problem than the spend it saved. A held workflow is resumed or
cancelled by you, and holding is recorded as an event like anything else.

#### When `Verify` fails, classify before retrying

Every guardrail above answers a failed iteration the same way: try again, with reflection, then give
up. That is wrong for at least half the failures, and Sengupta et al.'s deployment report names why —
**a contract that admits two readings is not fixed by another implementation attempt.** Their harness
routes every failure through a four-way arbiter first, and each class has a different corrective
action:

| Class | Means | What Locus does |
| --- | --- | --- |
| **Bug** | the implementation violates a clear requirement | retry the iteration, and promote the failing check into the task's regression set |
| **Spec gap** | the requirement omitted necessary behavior | back to the planning module as an amendment — a *new* task for the delta, since the original may be Done |
| **Noise** | environmental or irrelevant — a flaky test, a CI hiccup | recalibrate the check; do not count the iteration against `max_iterations` |
| **Ambiguity** | the requirement admits several valid readings | refine the requirement, then restart — never retry the implementation |

Two consequences worth having on purpose. **Noise stops burning the iteration budget**, which today
it does silently — three flaky failures and a workflow is dead at 8 iterations having attempted the
work five times. And **spec gaps and ambiguity leave the workflow entirely**, which is the only path
that reaches the thing actually broken; the two-reader test already in the planning audit is the
mechanism that resolves an ambiguity once it is routed there.

The arbiter is an agent with a bounded job, and its classification is a column on the iteration — so
**spec-gap rate and ambiguity-detection rate are queries**, and a workflow that keeps producing spec
gaps is visibly a planning problem rather than a builder problem.

**Reviewer agents need no special machinery.** A reviewer is an ordinary agent with read-only tools,
wired to a `Gate` that triggers on task completion — roughly one reviewer per three or four builders.
The `Gate` and `Verify` nodes exist so the pattern is expressible without hand-wiring.

**Role contamination is refused at compile time.** One agent definition may not hold both the builder
and the tester role in one workflow, and the reviewer may not be the implementer. This is the rule the
whole independence regime rests on, so it belongs in graph validation beside cycles and missing
`verify` — not in a convention someone follows. A verifier that wrote the code inherits its
assumptions, and a graph that quietly allows it produces reviews that agree with everything.

**Two regimes, and they catch different things.** Conflating them is why review sometimes finds
nothing:

| Regime | Mechanism | Catches | Cannot catch |
| --- | --- | --- | --- |
| **Independence-based** | one agent implements, another writes tests — both from the requirement, neither seeing the other | contract violations, behavioral bugs | anything the requirement never said; shared model blind spots |
| **Attention-based** | one model directed into separate reviewer roles — security, architecture, product, QA | structural, product, and UX problems | it is not independent, so a shared misreading survives |

**Locus can enforce independence structurally**, which is the part most setups only ask for politely:
separate containers, separate sessions, no shared conversation, no shared short-term memory — all
already true — plus the one the git model makes possible, **the tester clones the branch at base**,
before the builder's commits. A verifier that cannot see the implementation cannot adopt its
assumptions.

Stated honestly, as the source does: this is structural independence, not formal. Two agents on the
same foundation model can share a blind spot, and an incomplete requirement produces an incomplete
test suite no matter how independent the author.

### Teams — the workflow *is* the team

There is no `Team` entity. A workflow already declares everything a team is:

| A team needs | The workflow supplies |
| --- | --- |
| A roster | its `Agent` nodes |
| A role per member | the `role` field on each `Agent` node |
| Who waits on whom | the edges between them |
| A shared task list | the board scope the workflow owns |
| Peer messaging | `locus mail`, available to every agent regardless |
| A lead | **the graph itself** |

Two things fall out of this that are worth having on purpose:

- **Board dependency edges are generated from the workflow graph.** When a workflow creates tasks, the
  edges it was drawn with become the tasks' `blocked_by` relationships. Dependencies are declared once,
  visually, rather than drawn on the canvas and then re-entered on the board.
- **The lead is deterministic.** Decomposition is what you drew, not what a model decided this run.
  That makes a run reproducible and debuggable, and it is why `Condition` nodes route rather than a
  model routing. The cost, stated plainly: a workflow cannot re-plan itself mid-run. If dynamic
  decomposition turns out to be needed, the fix is an agent that *authors a workflow* and submits it
  for goal approval — not a model in the execution path.

### Two pipelines, deliberately different weights

```
AGENT                              WORKFLOW
markdown + frontmatter             canvas (nodes + edges)
   │ parse, validate tools            │ validate  cycles, unresolved handles,
   ▼                                  │           missing verify, unreachable goal,
agent_defs ─┬─ frontmatter JSONB      │           loop with no termination,
            │                        │           role contamination
            ├─ body       text        ▼
            └─ version                workflow_defs ─┬─ graph   JSONB, as authored
   │ materialize (at run start)                      ├─ spec    JSONB, executable
   ▼                                                 └─ version
/locus/config/agents/<name>.md        │ execute
  ~ an identity transform             ▼
                                      run supervisor walks the spec
```

Postgres stays the single truth for both. For workflows, `graph` is what the canvas reloads and `spec`
is what the supervisor reads, produced together so they cannot disagree. For agents there is nothing
to disagree about — the file *is* the definition.

**Live overlay.** Opening a running workflow shows its graph with run state painted on it: which node
is executing, which `Verify` passed or failed, which iteration the loop is on, tokens and wall-clock
per node. The agent event stream shows work; the canvas is the map. Both read the same normalized events, so
there is one source and two renderings.

### Agent CLI (`locus`, inside the container)

```
locus memory note add|replace|remove          the bounded core tier; over-cap writes ERROR, never evict
locus memory recall <query>                   the store tier: structured + FTS + vector
locus memory write --scope … --why …          a fact, with provenance attached automatically
locus memory forget <id>                      traced back to its writer, then deleted
locus mail send|list|read|reply|drain|wait   agent↔agent, Rust-native. `wait` blocks with a
                                             15-minute default timeout, then returns empty —
                                             it sets `waiting`, so it never reads as idle
locus task list|show|move|assign|comment     board, agent-driven
locus wiki search|read|write|history         wiki, with revisions attributed to the run
locus wiki ingest <path|url>                 read a document into typed pages; flags contradictions
locus wiki query <question>                  synthesize an answer, filed back as a synthesis page
locus wiki lint                              orphans, broken links, missing entities, unsourced facts
locus lsp def|refs|hover|symbols|rename       semantic navigation over this run's clone
locus debug start|break|step|stack|vars|eval  DAP; the session lives in the core, keyed by run
                                             `break --log` is a logpoint: prints, never stops
locus browse open|click|fill|assert|screenshot  the project browser, own context per run
locus browse record|console|network          flow recording, and the text a page produced
locus agent invoke <name>@<version>          run a nested agent: own container, own clone
locus svc up|down <name>                     start a project service container; agents get no Docker socket
locus ask <question>                         escalate to the human via the Chat pane; blocks
locus run status|artifacts                   this run's own state
locus handoff <agent> --why …                transfer ownership: ends this session, opens the
                                             successor's on the same task and branch
locus artifact put <kind> <path>             publish a plan, diagram, or image for review
locus artifact get <id> [--for-context]      read one back; --for-context returns OCR text or a
                                             downscaled frame, never the raw bytes
locus artifact comments                      feedback left on your artifacts, threaded
locus tools list|docs <name>                 marketplace docs for allowlisted CLIs
locus lint [--changed|--only NAME]           run this project's linters; the one extension
                                             no harness reads. Called by the agent before it
                                             commits, and by a workflow's `Verify` node —
                                             never from a hook, which would tax every tool call
```

`--json` on every command, because the caller is usually a model — compact, never pretty-printed, and
key-packed for uniform tables past a row threshold, which runs 50–60% smaller than minified JSON.

### Marketplace

A git repo of manifests, one per CLI:

```toml
# index/amq.toml
name    = "amq"
summary = "File-backed message queue between agents"
install = { brew = "amq", cargo = "agent-message-queue" }
verify  = "amq --version"
docs    = "docs/amq.md"        # injected into agent context when allowlisted
caps    = ["agent-messaging"]
```

Locus resolves an agent's `tools` list against the index, bakes installs into that agent's image, and
injects a **catalog** of them — name plus one line, roughly 15 tokens each — with every body fetched
on demand through `locus tools docs <name>`. **Nothing about a tool is loaded before the agent decides
to use it.**

This is the same move the field made for MCP under three different names — Anthropic's Tool Search
Tool, Cloudflare's Code Mode, the MCP-code-execution pattern — all versions of *stop loading tool
definitions you aren't using*, against a reported 55K+ tokens of schema consumed before work begins.
Reaching it from a CLI is a catalog line and a `docs` verb rather than an architecture.

The line an agent needs to *choose* a tool is short: what it does and when to reach for it. The page
it needs to *use* one — flags, output shape, examples — is only worth its tokens once the choice is
made. Fifteen allowlisted tools cost about 225 tokens instead of 3,000, and the difference is
recovered by any agent that actually reads a page.

**Installation stays eager, deliberately.** A tool absent from the allowlist is absent from the image,
because that is a privilege boundary rather than a context decision. Just-in-time applies to what an
agent *knows*, never to what it *can reach*.

**Those blurbs are worth iterating on.** Anthropic reported a **40% cut in task completion time** from
having an agent evaluate and rewrite tool descriptions — which makes a manifest's `summary` and `docs`
a tuning surface rather than boilerplate. The index is git-backed, so a better description is a commit,
and the event store already holds what it would be measured against: how often a tool was reached for,
and whether those runs passed. A tool not in the allowlist is not installed, so the agent
cannot reach for it.

The marketplace splits across two milestones, because agents need the *index* long before they need
the *installer*: reading manifests to validate an agent's `tools` list and inject docs is cheap and
lands at M4; image baking and install land at M8.

**Until M8 the index is a local directory of manifests**, read from disk. Where it is hosted, how it is
pinned, and who is trusted to publish into it are questions the installer makes real and the resolver
does not — so they are answered then, not now.

### Navigation — seven categories, one address space

Seven top-level categories, on a rail. The split is **by what you are doing**, not by what the data is —
which is why agents and workflows appear twice: operated in one place, authored in another.

| Category | Holds | The question it answers |
| --- | --- | --- |
| **Dashboard** | **Inbox** — my queue; **Status** — the numbers, at a glance | what I need to do, and what I need to know |
| **Plan** | the planning module — interviewer, researcher, auditor; specs and the recommendation | what should we build, and is it understood |
| **Develop** | editor, diff review, file tree, search, git — branches, PRs, merge-back, and terminals I drive myself | the hands-on work, mine |
| **Automate** | agents and their sessions, the board, running workflows, schedules | what is assigned, and what is running |
| **Review** | telemetry, run history, cost, verify and spec-gap rates, artifacts and walkthroughs | what happened, and was it any good |
| **Workshop** | settings, harnesses and model tiers, the eight extension types, agent definitions, the workflow canvas, the marketplace | the tools themselves |
| **Wiki** | typed pages, ingest, contradictions, the wikilink graph | what do we already know, and what disagrees |

**Dashboard is the category; `Inbox` is what the rail calls it.** The category holds two views and the
rail cannot show both, so it is labelled by the one that carries the badge. Internally the category key
stays `dashboard`, because a category called Dashboard containing a view called Dashboard is exactly the
blur the two view names exist to prevent.

**A session lives with the thing it belongs to.** A session is one agent's thread of work, so its home
is Automate, beside the agent it belongs to and the board task it serves. Dashboard is where you find
out that something needs you; Automate is where you go and watch it. The same rule places the other
two session kinds without argument: the planning conversation lives in **Plan**, because its purpose
is the spec, and a terminal you drive yourself lives in **Develop**, because it is not an agent's
session at all.

**Dashboard is the category defined by whose it is, not by what it holds.** The other five are named
for an activity; Dashboard is *mine* — the things I need to do, and the information I need in order to
do them. That is why Inbox and Status sit together despite looking like different kinds of thing: one
is my queue, the other is my context for working it.

Its two views are named separately for a reason. **Status is the at-a-glance half** — is anything on
fire, what is running, what did today cost — and it deliberately does not grow a query tool, because
that is Review's job. A category called Dashboard containing a view called Dashboard would guarantee
the two blur.

Which sets the rule for how much happens there. **A decision resolves in place; work routes out.**
Approving a goal, accepting or rejecting a proposed learning, answering a `locus ask`, waving through
a `Gate` — those are answers, and making me travel to give one is the cost this category exists to
remove. Anything that is *work* — editing the spec the gate was about, fixing the code the review
found — opens where that work lives, and Dashboard hands me the locator rather than growing a second
copy of the surface.

**Dashboard is now; Review is after.** Status says whether the system is healthy at a glance; Review is
where you dig into a run that was not. Keeping them apart stops Status growing into a query tool and
stops the history growing a live view that nobody watches.

**Automate operates; Workshop authors.** An agent you are assigning work to and an agent definition
you are editing are different activities on the same object — one is a thing doing work, the other is
a document under version control. Same for workflows: the canvas is Workshop, a running execution with
its live overlay is Automate.

**Workshop is where the meta-harness actually lives.** Skills, rules, commands, hooks, output-styles,
linters, agents, base-context — authored once, materialized into every runtime. That is the product's
central claim, and it deserves a place rather than a settings page.

#### The inbox

Its home is Dashboard, and **its count is on the rail**, so silence is legible without navigating. That
is the whole property worth protecting: a session working normally puts nothing in it, and you should
be able to see that from anywhere.

Every item resolves to something — a `Gate` opens the artifact it waits on, a `locus ask` opens that
session's chat, a contradiction opens both wiki pages. An item that only reports that something
happened is a notification, not inbox work.

#### One window. Project is a filter, not a boundary

A window per project would rebuild the thing this design exists to remove: one inbox per window, one
board per window, one place per window to notice an agent has been idle for an hour. Every layer
beneath the UI was built to be a single surface across every runtime; sharding at the last layer
throws that away.

So **project is a scope control, defaulting to all**. You filter, never switch — switching means
leaving somewhere that was still running. Cross-project questions are the useful ones: everything in
Reviewing, every idle agent, every guardrail trip today.

The cost, stated: every project-scoped surface carries a scope control, and every cross-project row
shows which project it belongs to. That is real work in the board and in Status, and it is cheaper
than a window per project.

Additional windows still exist for detaching a pane onto a second monitor. That is a display choice,
not the organising principle.

#### Sessions do not all fit, so most are strips

One ACP conversation per run is chosen for observability, which creates the obvious problem at ten
sessions. **Automate** holds **one to four focused panes**; everything else is a strip entry — the
minimize-to-tile behaviour already specified, made the default rather than the exception. The strip
persists across categories, because walking away from Automate is not a reason to lose sight of what
is running.

A strip entry carries what you need to decide whether to look: project, agent and role, its task,
status, the tool it is running, tokens so far, and an **idle icon** when the guardrail says so. Sorted
by needs-attention then recent activity — never by project and never alphabetically, either of which
would put the same session in the same place whether or not anything is happening to it.

Promoting an entry demotes the least recently focused pane. Nothing is closed by promotion: a session
you stopped watching is not a session you ended.

#### One address space, so there is one resolver

Every addressable thing has a locator:

```
locus://<project>/session/<id>[/run/<id>]
locus://<project>/task/<id>       artifact/<id>       page/<slug>
locus://<project>/workflow/<id>[/execution/<id>]      agent/<name>@<version>
```

This is the structural decision, and it is what lets project be a filter at all: the project is a path
segment, so a cross-project view is a query over locators rather than a different kind of screen. The
command palette, global search, inbox items, board-card links, artifact comments, notification deep
links, and a detached window's identity are otherwise seven navigation paths that drift apart; against
one locator they are one resolver with seven callers. `Cmd-K` resolves a locator, `Cmd-P` searches for
one, and back/forward per window is a stack of them.

A locator also crosses categories cleanly, which the seven-way split needs: the same session opens as a
pane in Automate, as the source of an Inbox item, and as a row in Review — one object, three contexts,
no duplicated navigation.

#### Three rules that keep it from sprawling

- **Detail opens in place.** A task, an artifact, a page opens as a sheet over the current category,
  not as a new category or a new window. You came from somewhere and you are going back there.
- **One viewer per kind, several entry points.** An artifact looks the same in Automate, in Review,
  and reached from the inbox — one component, because a diff that renders differently depending on how
  you got to it is two components that will disagree.
- **The category list is closed.** Seven, and a new surface joins one of them rather than adding an
  eighth. A rail that grows is a rail nobody reads. The count was six until the Wiki was given its own
  entry; the rule is what stopped it becoming eight at the same time.

### Frontend and IPC constraints

Three rules that are cheap to follow from the start and expensive to retrofit.

**Channels for streams, events for notifications.** Tauri's own documentation is explicit: the event
system "is not designed for low latency or high throughput situations" and works by evaluating
JavaScript; `tauri::ipc::Channel<T>` is what they use internally for child-process output. Every
high-frequency path here is a Channel:

| Path | Mechanism |
| --- | --- |
| Human-terminal PTY bytes | `Channel<&[u8]>` — never an agent session |
| Normalized session events, including token deltas | `Channel<Event>` |
| LSP diagnostics and semantic tokens | `Channel<T>` |
| "a run finished", "a task moved", "a guardrail tripped" | `emit` — low frequency, many listeners |

Coalesce per pane on an animation-frame tick regardless of mechanism. A thousand small sends per
second will drop frames whatever the transport.

**Never two webviews in one window; any number of windows.** Tauri v2's multiwebview — several
webviews inside a single window — is behind an unstable flag and less mature than Electron's
renderer-per-window model. Multi-*window* is not: it is ordinary and well-supported.

So detachable panels are built as **additional windows, one webview each**, running the same Solid app
in detached mode. This costs almost nothing here, because panes are already views over runs and the
Rust core plus its event bus is already the source of truth — a detached window subscribes to the same
bus. There is no shared JS state to synchronize because there is no shared JS state. Layout inside a
window is a JS-side pane manager over one webview.

**The component library is deliberately small.** Every large surface either is bespoke or brings its
own DOM, so the library only has to cover chrome:

| Surface | Provided by |
| --- | --- |
| Pane manager, tiles, minimize-to-tile, file tree | bespoke — core product, not chrome |
| Workflow Canvas | `solid-flow` |
| Agent editor | a Markdown editor — CodeMirror, already present |
| Editor and diff | CodeMirror 6 |
| Human terminals | xterm.js |
| Board, wiki, dashboard | bespoke |
| Dialogs, context menus, tabs, tooltips, combobox, toast, palette | **Kobalte**, styled via **shadcn-solid** copied into `src/ui/` |
| Long lists | `@tanstack/solid-virtual` |

Kobalte ships Combobox, Context Menu, Tabs, Dialog, Tooltip, and multi-region Toast. It ships **no**
resizable split panes and no tree — correctly, because both of those are the pane manager and the file
tree, which are product here rather than components. shadcn-solid is copied in rather than depended
on, so nothing visual is version-locked, and it sits on Kobalte so there is one primitive layer.

**Keyboard capture is a human-terminal problem.** xterm.js remains for hands-on work; agent sessions
render ACP events and do not require terminal keystroke fidelity.

- Option-as-Meta and pre-processing hooks are xterm.js configuration — `macOptionIsMeta` and
  `attachCustomKeyEventHandler` — not code to invent.
- **Cmd chords are the real fight.** AppKit consumes them via the menu bar. Ship no default menu, or
  one whose items carry no key equivalents, and register in Rust only the accelerators actually
  wanted.
- **IME composition and dead keys need testing, not design.** Budget the time; there is no clever
  answer, only trying it in each webview.

### Editor

CodeMirror 6 used directly, no wrapper interface.

- `@codemirror/lsp-client` — completion, hover, signature help, format, rename, jump-to-definition,
  find-references, diagnostics, and its `Workspace` abstraction for multi-file.
- `@codemirror/merge` — `MergeView` with `collapseUnchanged` and per-chunk revert controls. This is
  the **primary** editor surface: reviewing what an agent changed.
- **The editor opens an ordinary clone.** For a **linked** repo that is your own checkout, where
  `git fetch locus && git checkout agent/<run-id>` is the motion you already know. For a **managed**
  repo Locus keeps one normal clone per project beside the bare remote and opens that. No worktrees
  anywhere in the design — a clone is what the git model already produces, and adding a second
  checkout mechanism would mean two ways to be on the wrong branch.
- Language servers are spawned and supervised **on the host** by the LSP supervisor, one set per
  project, shared across panes. Agents' containers do not run language servers.

**One editor, two zoom levels.** The side pane beside an agent and the full-window editor module are
the same CodeMirror components at different sizes, sharing one keymap, one theme, one LSP client. A
second editor for the "real" module would mean two of each and muscle memory that breaks whenever you
move between them — and would re-import the whole extension-host cost for the surface you use *least*
in an agent-first IDE. File tree, tabs, splits, find-in-files, go-to-symbol, and multi-cursor are all
either native to CodeMirror or chrome built once and reused at both sizes.

Declined on purpose, restated so they are not rediscovered later: **no debug UI** — no gutter, no
variables pane, no step controls, because `locus debug` serves the side that needed it. Accepted gaps:
no VS Code extensions, and Lezer grammar
coverage thins out in the tail — Odin and GDScript have none. Mitigation when it bites is LSP semantic
tokens for color plus tree-sitter-WASM decorations for structure.

**Debugging is not an editor feature here.** The DAP client lands in the core because *agents* need to
debug, and it is reached only through `locus debug` in a container. The editor gets no debug UI — no
gutter, no variables pane, no step controls — so CodeMirror's lack of one costs nothing. That was the
one real gap in the CodeMirror trade, and it closed by moving the capability to the side that needed
it.

---

## Phased roadmap

Each milestone ships a runnable `verify:`.

### M0 — Specification and spikes  ← the immediate deliverable

Write the specs, then answer the questions that could invalidate them.

**The specs live in `.specs/`, one directory per feature**, each holding a `spec.md` (purpose, the
PLAN.md section that governs it, its contract, falsifiable acceptance criteria, dependencies, and what
is genuinely open) and a `tasks.md` (numbered steps, each with a runnable `verify:`). Every milestone
below is decomposed that way.

**This document stays the architecture.** A spec cites the section that governs it rather than
restating it — one source for a decision, not two that drift.

- `harnesses/*` — **all eleven written**, every one of the eight extensions declared with a `via`
  strategy and every downgrade carrying `weaker_than_native`. One is UNVERIFIED against a running
  binary — `dsh`, not installed here and not present in `local-dx` — and says so in the file. What
  remains is confirming it against its harness, which is Spike 1's other half. `hermes` was removed
  ACP-only: it ships no first-class ACP mode and is declared out of support.
- Fill in the stub `AGENTS.md` at the repo root
- **Spike 1** `spikes/01-sandboxed-harness/` — does a harness run in a container built from
  `locus/base-<harness>`, authenticated without holding a long-lived secret, against its own clone,
  emitting a parseable event stream? This is the highest-risk unknown in the whole design: if harness auth cannot be injected
  without baking a secret into an image, the sandbox model changes.
- **Spike 2** `spikes/02-editor-embed/` — CodeMirror 6 + `@codemirror/lsp-client` against a real
  `rust-analyzer` inside a Tauri window, plus a `MergeView` over a real git diff.
- **Spike 3** `spikes/03-workflow-canvas/` — `solid-flow` with custom typed nodes, a graph that
  round-trips through JSONB unchanged, and a loop construct whose termination is checkable at compile
  time. Its reputation on Context7 is Medium, not High, so confirm it carries this weight before M4
  depends on it.

```bash
# every harness declares all eight extensions, each with a known `via` strategy
locus harness lint          # refuses an undeclared extension or an unknown strategy
```

`verify:` all three spikes produce a written answer in their directory; every `.specs/*/` holds both a
`spec.md` and a `tasks.md` and every task row carries a runnable command;
`locus harness lint` passes for all eleven.

### M0.5 — Desktop UI on fixtures

**Historical record.** M0.5 delivered the v1 shell and fourteen fixture screens before the runtime.
The v1 handoff was removed when v2 replaced it; its completed contracts remain under `.specs/` only as
baseline history. New desktop work follows the M0.6 v2 reconciliation below.

Built ahead of the runtime on purpose. The alternative — a screen arriving with the milestone that
makes its data real — spreads one coherent visual system across five milestones and re-decides it each
time. The cost is the honest one: fixture shapes are invented before the schemas exist.

- **Design system** — the token table as CSS custom properties; Inter, JetBrains Mono and Phosphor
  **vendored rather than linked**, because a desktop app must work offline; `pulse` and `blink` and
  nothing else animating; and rulings on the four states the handoff says were not drawn — hover,
  pressed, focus, and the loading/empty/error cases
- **`src/ui/`** — shadcn-solid over Kobalte, copied in and owned here, never depended on
- **Shell** — title bar, the `locus://` locator bar as the `Cmd-K` surface, the seven-item rail with
  its inbox badge, the per-category tab bar, and the running-agent strip as a footer on every screen
- **Navigation** — the view/category/locator map and the per-category tab sets, with `agents` as a
  drill-down of Extensions rather than a tab. **The locator resolver lands here**, not in M1:
  retrofitting one address space onto navigation that already exists costs more than starting with it
- **Fourteen screens** — Inbox, Status, Plan, Develop, Automate Kanban, Automate Agents, Review
  Telemetry, Review Runs, Review Artifacts, Workshop Extensions, Workshop Agent-definitions, Workshop
  Workflow, Workshop Harnesses, Wiki
- **Real renderers, not the mockup's hand-drawn SVG** — the runs-by-hour bars, the telemetry sparkline
  and facet tracks, the wiki graph, the canvas edge layer

**Fixtures are derived, never invented.** Their types come from the eight schemas and the canonical
event vocabulary rather than from what a screen happens to need; each module names the schema and the
command that will replace it; and the Harnesses and Extensions screens compute theirs from
`harnesses/*.toml`, so those two are correct on the first day and stay correct without an edit.

Historical verification reached every v1 screen, derived Harnesses data from the TOMLs, and enforced
keyboard focus and a clean `pnpm build`. The v2 verification inventory is `.specs/design-v2/tasks.md`.

### M0.6 — v2 desktop reconciliation

The v2 handoff replaces the shell, expands the fixture set to thirty-one screens, and introduces
Providers, CLI tools, project scope, plan decomposition, dispatch, and workflow Governance. It also
ships Dark and cool-neutral Light themes through semantic roles. The contract is `.specs/design-v2/`;
theme values and regression requirements are `.specs/theme-system/`.

`verify:` the 31-screen fixture inventory, provider-secret redaction, selected-project navigation,
queue-stop semantics, and both theme visual/contrast matrices pass their task commands.

### M1 — Core runtime

Daemon, store, registry, containers, and ACP conversations.

- `locus-postgres` lifecycle, migrations (`sqlx`), and **backup/restore**: `locus backup` dumps the
  eight schemas and the artifact blob tree together, nightly and before every migration, retaining
  seven dailies and four weeklies. `locus restore --drill` restores into a scratch database and
  asserts row counts against the source — because a backup nobody has restored is a belief, not a
  backup, and this is the one piece the deferral table calls non-deferrable
- **ACP client** on the `agent-client-protocol` crate — the only agent-session transport
- Harness registry and TOML schema validation, refusing any entry that leaves an extension undeclared
- **Settings → Harnesses**: the tier-to-model grid, populated from `[models].list_argv` where a harness
  can enumerate and free text where it cannot, stored in `core.settings`. Unset tiers pass no flag, so
  a freshly registered harness runs on its own default rather than not running
- **Materializers**, **byte-deterministic**: sorted order, no timestamps, no run id — two runs of the
  same agent produce an identical config tree, which is what makes the prompt prefix cacheable. The
  four generic strategies (`dir`, `merged-into`, `listed-in`, `entries-in`), plus
  the `plugin` host and **one** real plugin — pi's, because a generated TypeScript extension is the
  furthest a harness gets from copying a directory, so it proves the contract at its hardest point
- **Agent definitions**: Markdown + frontmatter, versioned, materialized per run. Because agents are
  not a graph, they land here rather than waiting for M4 — the product is usable four milestones
  earlier than the previous draft implied
- Container supervisor (`bollard`), image build, mount and credential injection
- **Session and run model**: sessions persist across runs, runs bounded by the container
- **Artifacts**: plan, diff, and walkthrough kinds, attached to sessions, with comment threads that
  route feedback back into the session. Text in Postgres and the blob tree under
  `/var/lib/locus/artifacts/` from the first artifact, so backup covers it from day one. Media kinds
  and the derived-representation cache arrive with the browser at M3.5
- Run supervisor: spawn → stream → normalize → persist → cancel
- Credential broker and egress policy, whichever shape Spike 1 settles on
- **Wire the M0.5 screens whose data M1 makes real** — Automate Agents and its transcript, the Inbox,
  Status, Workshop Extensions and Workshop Harnesses. Each swaps its fixture accessor for a Tauri
  command; the shell, the rail, the navigation and the locator resolver already exist and are not
  rebuilt. Screens whose backend is still ahead of them keep their fixtures and say so on screen
- JS-side pane manager over **one webview per window**, one window holding every project, ACP event
  panes for agent sessions, and the project-labelled session strip as the default home for anything
  not focused; xterm.js remains only for human terminals
- All streaming over `tauri::ipc::Channel`, coalesced per pane on a frame tick, from the first commit
- **CI for Locus itself** — `cargo test`, `cargo clippy`, the harness lint, and the materialization
  smoke test on every push. Cheap now, and the smoke test is the only thing standing between a harness
  release and a silent non-load

`verify:` two agents from different harnesses run concurrently as ACP conversations in their own
containers against the same repo, and a third on a **different project** appears in the same tile strip
beside them; their events are indistinguishable downstream; every event lands in Postgres;
both minimize to informative tiles. A canary skill and a canary rule materialize into **every**
registered harness and the agent can see both — including pi, where they arrive as generated
TypeScript rather than as files. Materializing the same agent twice produces byte-identical trees:
`diff -r` is empty, and the run's `usage.cache_read` share is the evidence it mattered. A terminal survives a `vim` session with
Option-as-Meta, Cmd chords reaching the app rather than the menu bar, and IME composition intact.

### M2 — Workspace

- File tree, CodeMirror editor pane, side-pane and full-window modes — **the same components at two
  zoom levels**
- LSP supervisor and multiplexing, host-side, serving the editor panes
- **`locus lsp` for agents** — the same Rust client, but running in the agent's container against that
  run's clone. Lands here because it shares the client with the editor's
- `MergeView` diff review over an agent's pushed branch, opened in an ordinary clone — your own
  checkout for a linked repo, Locus's working clone for a managed one
- Search across the project
- Exercise the editor on **all three webviews** — WKWebView, WebView2, WebKitGTK. This is Tauri's
  real tax and the only place it lands hard

`verify:` open a file, get real completions and diagnostics, review an agent's diff, accept a hunk —
on each of the three webview engines. Separately, an agent in a container resolves a symbol's
definition through `locus lsp` and gets the same answer the editor does.

### M3 — Coordination, memory, and mail

The shared-services milestone. Everything here is Rust in `locus-core`, surfaced identically to every
harness through the `locus` CLI.

- **Memory**: capture via `locus-hook` into `memory.event`; the store with role ACLs, provenance and
  hybrid recall; the 800-token catalog injected at `SessionStart` where the harness supports it and
  materialized into `[layout].context` where it does not; the keeper, promotion checks, and decay
- **Tool-output compaction**: a `PreToolUse` hook in the base image that rewrites verbose commands and
  compacts their results before they reach context — `rtk`'s mechanism, shipped once and reaching every
  harness because Locus already materializes hooks
- **Mail**: threads, delivery, `wait`/`drain` semantics, `locus mail` verbs
- **Handoffs**: the payload artifact, `handed_off_from` on the session, and the guardrail's
  kill-and-reassign rewired to produce one instead of dropping the work on the floor
- Repo manager: bare local remote per project, per-run clones with `--reference` against a shared object store
- **Agents push to a local git remote; you `git fetch && git checkout` from your normal repo.**
  Sculptor's pattern, and better than bind-mounting a workspace into the human's editor: it
  keeps Locus out of your editor, your merge tool, and your shell, and it means reviewing an agent's
  work uses the git you already know
- Merge-back: each agent merges its own branch when it is done, and a conflict it cannot resolve
  becomes an inbox item
- Run guardrails: `max_iterations`, forced reflection before retry, token budget with auto-pause, kill
  and reassign when stuck, idle detection surfaced on the tile

`verify:` three agents work the same repo concurrently from their own clones; two exchange mail; one
writes a memory that a **different harness** later recalls; all three push branches that merge back or
report a structured conflict.

### M3.5 — Agent capabilities: debug and browser

Small, and it belongs before the canvas because a workflow that cannot verify a UI or inspect a stack
is a workflow that escalates to you constantly.

- **DAP client in Rust** → `locus debug`. Adapters run in the agent's container, sessions are held by
  the core and keyed by run, logpoints are the default path, and a paused program suppresses the idle
  guardrail
- **Browser container per project** → `locus browse`. Playwright driving Chromium on the project
  network, reaching the agent's app on `$LOCUS_PORT`, **one browser context per run** so concurrent
  agents cannot see each other's pages, cookies, or storage. No egress unless a project grants it.
  `assert` exits non-zero with structured JSON so `Verify` can gate on it
- **Image and recording artifacts** from `locus browse`, attached to the session and the board card,
  so a UI change is reviewable without checking out the branch — stored once for you, derived on
  demand for a model, OCR before pixels

`verify:` an agent sets a breakpoint, inspects a variable, and fixes a bug it could not have found
from print statements — and sitting at that breakpoint for five minutes trips no idle guardrail. Two
agents drive the shared browser at once without seeing each other's cookies or pages. A second agent
changes a page, records the flow, and the recording appears as a commentable artifact — and a comment
left on it reaches the agent. `locus browse assert` fails a workflow's `Verify` node on a real
regression and passes on a real fix.

### M4 — Workflow Canvas

Agents already work — they are Markdown and shipped in M1. This is where *orchestration* becomes
authorable, and where the product's character arrives.

- `solid-flow` canvas, the workflow node types, typed handles, graph validation
- Compile pipeline: `graph` JSONB → `spec` JSONB → versioned `workflow_defs`, round-tripping exactly
- **Loop execution**: pick → act → validate → commit → reset, with memory carrying across resets
- **The Ralph preset** — `locus ralph --goal … --verify …` for a loop without opening the canvas, and
  the same preset droppable onto the canvas where it expands into ordinary nodes
- **Goal as approval gate** — a person approves the goal before the loop is allowed to run
- **Guardrails**: `max_iterations`, forced reflection before retry, kill-and-reassign, idle detection,
  optional wall-clock ceiling, optional token budget with auto-pause at 85%
- **Deterministic routing** — `Condition` nodes decide, never a model. No LLM in the orchestration path
- Tool manifest **index and resolver** (not the installer) so agent `tools` lists validate
- Live overlay: the graph re-rendered with per-node state, iteration count, verify results, and spend

- **Roles and dependencies** on `Agent` nodes and edges — the workflow *is* the team definition, and
  the tasks it creates inherit their `blocked_by` edges from the graph

`verify:` draw a workflow that loops a builder, a tester, and a reviewer against a goal, with the tester
depending on the builder; approve the goal; it runs unattended, the tester starts the moment the
builder's task completes, and it stops on `max_iterations` when the goal is unreachable with the
overlay showing which iteration tripped which guardrail. Reopening reproduces the graph exactly as
authored. A loop with no termination condition is refused at compile time, not at run time, and so is a graph
that hands one agent both the builder and the tester role.

### M5 — Project management

- Board: fixed columns, tasks, drag, agent-driven transitions, `blocked` as a status
- **Dependency edges with auto-unblock** — completing a task clears the `blocked` status on its
  dependents, and a waiting agent picks them up without a human noticing first. Edges are generated
  from the workflow graph that created the tasks
- **Evidence links** on every agent-made transition, citing the run and the events that justify it
- Wiki: **typed pages** (source, entity, concept, synthesis, decision, overview), **ingest** via
  `markitdown`, **contradiction flags raised at ingest time** as board cards, `locus wiki lint`, a
  `solid-flow` graph view over `[[wikilinks]]`, GUI editor, revisions, `pgvector` search
- **The planning module** — the three-agent sequence, over ACP; approval lands tasks on the board
- **Reflection review queue** — the human gate where an agent's proposed learnings are accepted into
  always-on project context or discarded
- **The calibration loop** — a retro agent reads the arbiter's failure classifications since the last
  pass and proposes exactly four kinds of change, each aimed at the class that produced it:

  | Recurring class | Proposal |
  | --- | --- |
  | Bug | promote the failing check into the project's regression set |
  | Spec gap | add a clause to the relevant specialization record |
  | Noise | recalibrate or quarantine the check that keeps failing for nothing |
  | Ambiguity | add a topic to the interview, or a rule to the reduction pass |

  Every proposal lands in the reflection review queue and **none applies without you** — the same gate
  the memory keeper's promotions pass through, for the same reason. This is what makes the system
  improve rather than merely repeat: a failure that only ever produces a retry teaches nothing, while
  a failure that changes a template is paid for once

`verify:` a failing check is classified by the arbiter, and a run that failed on noise does not spend
an iteration; a recurring spec gap produces a proposed clause on a specialization record that waits
for a human rather than applying itself. An agent moves a task to Done and cites the run and the test
output that justifies it; a
blocked task auto-unblocks and is picked up without human input; a wiki page edited in the GUI is read
back by an agent in a container; ingesting two documents that disagree produces a contradiction card
rather than two quietly conflicting pages.

### M6 — Automation and discoverability

- Schedules: cron → workflow, with executions recorded against their verify result. **Overlap is
  skipped, never queued** — if the previous execution is still running the firing is recorded as
  skipped and dropped. A queue means a slow workflow silently builds a backlog that runs all at once
  when it finally finishes
- Dashboard: runs, spend, **cache rate**, **the tool-payload offender ranking**, verify pass rate,
  guardrail trips, board throughput, and the harness-level metrics the arbiter makes available —
  **spec-gap rate**, **ambiguity-detection rate**, **average iterations per task**, and **review-gate
  precision**, which separate a bad builder from a bad specification. Plus **agent trust** —
  that agent's verify pass rate over its last 20 runs, discounted by guardrail trips, by artifacts a
  human rejected, and **by tokens spent per passing run**. Every term is already a row, so it is a
  query rather than new instrumentation
- Discoverability: command palette and global search across code, wiki, tasks, and run history —
  both resolving locators, so neither is a second navigation system

`verify:` a scheduled workflow runs unattended overnight and reports green from its verify command;
a second one trips its token budget, auto-pauses at 85%, and notifies rather than draining.

### M7 — GitHub

Version control, CI/CD, and PRs only.

- `gh` + `gix` in core: branch, PR open/review/merge, CI status
- CI status surfaced on the board and dashboard
- **CI babysitter** — a failing pipeline pulls its logs, feeds them to an agent, retries a bounded
  number of times, then escalates. Sculptor and Agent Orchestrator both ship this and it is the single
  most-cited reason people leave a run unattended

**GitHub Issues, linked by explicit action in either direction.** No background sync, no polling, no
conflict resolution — the link is established by a person, once:

- **Attach an existing issue** to a Locus task → imports its title, body, and labels at that moment
- **Create a GitHub issue** from a Locus task → pushes it out and records the link
- Either way the task carries the issue number and URL, and the PR closes it with `Fixes #142`

Nothing syncs in the background. That is the point: every tracker integration that tries to keep two
systems continuously equal ends up owning a conflict-resolution problem nobody asked for.

**Agent-authored PRs**, as a first-class flow rather than "the agent ran `gh pr create`". This reuses
the artifact comment machinery — a PR review comment and an artifact comment are the same thing
arriving from two places:

- **Open** — the agent's branch becomes a PR with a generated description written from the session's
  goal, the tasks it closed, and the evidence it collected. Screenshots from `locus browse` attach
  here, so a UI change is reviewable without checking anything out
- **Slice** — a large change is split into several reviewable PRs rather than one that nobody reads.
  PR size is the strongest predictor of whether a review actually happens
- **Self-review first** — the agent reviews its own diff and fixes what it finds before asking you.
  You see the second draft, not the first
- **Respond to review comments** — a comment on the PR routes back into the session that authored it;
  the agent pushes updates and replies. This is the half most tools miss, and it is where the human
  time actually goes
- **Propose merge resolutions** — a conflict comes back as a proposed resolution to accept or reject,
  not as a problem handed to you

`verify:` an agent's pushed branch becomes a PR with a description, evidence, and screenshots; a review
comment left on GitHub reaches the authoring session and produces a follow-up commit; a deliberately
broken build is fixed by the babysitter within its retry budget, or escalates cleanly when it is not.

### M8 — Marketplace installer

The index landed at M4; this is the half that puts tools in images.

- Image baking, install methods, allowlist enforcement at container build, docs injection

`verify:` adding a CLI node to an agent installs that tool in the agent's image and its docs appear in
the agent's context.

---

## Critical files this plan creates

Nothing exists yet, so all of these are new:

- `.specs/<feature>/{spec.md,tasks.md}` — one directory per feature, every milestone; M0's whole output
- `harnesses/*` — one entry per harness, file or directory; what keeps harness names out of core
- `crates/locus-core/src/materialize/` — the generic strategies and the plugin host, naming no harness
- `crates/locus-core/` — registry, adapters, supervisors, store, **and every shared service:
  `memory/`, `mail/`, `board/`, `wiki/`, `telemetry/`, `tools/`**
- `crates/locus-cli/` — the in-container agent CLI; a thin socket client with no logic of its own
- `apps/desktop/` — Tauri + SolidJS. `workflow-canvas/` (node components, compile client, overlay),
  `panes/` (the pane manager), `ui/` (shadcn-solid components copied in, owned here), plus the shell,
  the seven-category navigation, and `fixtures/` with the types the screens read before M1 wires them
- `migrations/` — the eight schemas
- `spikes/01-sandboxed-harness/`, `spikes/02-editor-embed/`, `spikes/03-workflow-canvas/`

Reuse rather than rebuild:

- **`agent-client-protocol`** (crates.io) — the ACP wire model in Rust, for the planning module.
- **Vibe Kanban** (BloopAI, now community-maintained after bloop shut down in April 2026) — the
  closest prior art for board-drives-agents UX, open source and free to read.
- **Sculptor** (Imbue, MIT) — container-per-agent and the local-git-remote handoff pattern.

- `~/Repos/local-dx/config/harnesses.json` — the field-by-field description of how each of seven
  harnesses reads each feature. The single most valuable input to `harnesses/*.toml`; read it before
  writing the first one.
- **`SamurAIGPT/llm-wiki-agent`** (MIT) — typed page namespaces, ingest-time contradiction flagging,
  the wiki linter, and the wikilink graph. Read it before writing the wiki schema.
- **Hermes Agent** (Nous Research) — the bounded-core memory mechanics: hard caps, refuse-don't-evict,
  frozen snapshot for prefix-cache preservation, agent notes separate from user profile.
- **Docker Sandboxes / `sbx`** — credential injection via a host proxy, and Open/Balanced/Locked
  network policy tiers. Closed source; a pattern to evaluate in Spike 1, not a dependency.
- `markitdown` (already installed) — non-Markdown ingest for the wiki.
- `~/Repos/local-dx/cli/dx-telemetry` — its normalization pass and the four Postgres-vs-SQLite
  differences it documents, all four of which will recur here.
- `amq` (`/opt/homebrew/bin/amq`) and `memsearch` — **read for their verb sets and scoping model
  only.** Neither is linked or shelled out to: both store data as files, which is precisely what
  putting memory and mail in Rust behind Postgres is meant to fix.

## Verification

**How Locus is tested: event-based.** Every run normalizes into `memory.event` regardless of harness,
capture path, or container — so a test is *"run this, assert these events appeared."* That works
identically across all eleven harnesses and needs no test-only instrumentation, because the substrate
is the one telemetry already requires. Unit tests cover the pure parts; everything above them asserts
on the event stream.

M0 is complete when:

```bash
# every feature has both halves, and no task is missing its runnable check
for d in .specs/*/; do
  test -f "$d/spec.md" -a -f "$d/tasks.md" || echo "INCOMPLETE: $d"
done
awk -F'|' '/^\| *[0-9]+ *\|/ { if ($(NF-1) !~ /`/) print FILENAME": "$0 }' .specs/*/tasks.md

# spike 1 answers the auth question in writing
cat spikes/01-sandboxed-harness/FINDINGS.md

# spike 2 answers the embed question in writing
cat spikes/02-editor-embed/FINDINGS.md

# spike 3 answers whether solid-flow carries the workflow canvas
cat spikes/03-workflow-canvas/FINDINGS.md
```

Toolchain is present and current: `docker 29.7.1`, `node v24.19.0`, `pnpm 10.26.2`, `cargo 1.97.1`,
`gh 2.97.0`. `tauri` CLI is **not** installed and M0 must add it.

## Risks

**Risk — harness credentials.** Spike 1 exists because this can invalidate the sandbox model. If a
harness will not authenticate from a runtime-injected, read-only credential, the fallback is a
short-lived credential broker on `/run/locus.sock` rather than a mounted file — more work, and it
must be known before M1 rather than during it.

**Risk — TUI-only harnesses are excluded.** A harness that insists on painting a full-screen
interface cannot be supported, because it does not expose the one ACP conversation per run that Locus
owns and renders. Deliberate, but real.

**Risk — a complete-looking requirement that is not.** Every gate in the design checks work against a
requirement, so a requirement that omits a behavior produces a green run, a passing adversarial suite,
a Done card with evidence, and a broken feature. Nothing downstream can catch it: this is a limit of
the shape, not a bug in it. The mitigations are the two the deployment report arrived at — a reduction
pass that removes speculative clauses so the real ones are visible, and specialization records that
carry a domain's hard-won clauses into the next spec — plus the arbiter making spec-gap rate a number
somebody looks at.

**Risk — prefix stability decays by accident.** Nothing fails when it breaks; the runs just get more
expensive, and the cause is whatever injection point was added last. Every future feature that puts
bytes in front of a model — a new hook, a status line, a richer catalog — is a chance to put something
mutable in front of something shared. The defence is that cache rate is on the dashboard and the
determinism check is in CI, so a regression shows up as a number rather than as a slow drift nobody
attributes.

**Risk — harness surfaces rot, in both halves.** Structured events come from session logs and hooks;
config layouts come from directories, filename suffixes, and config keys. All of it is the harness's
private business and changes between releases — `opencode` reading `command` and not `commands`, or
copilot ignoring a plain `.md`, are the kind of difference that fails **silently**: the tree is
written, the run starts, and nothing loaded. `dx-telemetry` has absorbed four dialects already, so
this is known recurring work rather than a surprise, and it is the price of not owning the harness.

The mitigation is a **materialization smoke test per harness**, run on registration and in CI: start a
run with a canary skill and a canary rule, and assert the agent can see both. That converts a silent
non-load into a failing test, which is the only reason the `emits` and `via` declarations are worth
writing down at all.

**Risk — DAP adapter coverage.** The client is one implementation, but every language needs its own
adapter and each must be installed in the agent's image — so `locus debug` is only as broad as the
adapter set baked per project. Node, Python, and Rust (CodeLLDB) are well served; the tail is not.
This replaces the earlier "no debugger" risk, which the decision to ship a DAP client retired.

**Risk — terminal keyboard fidelity.** A webview terminal feels wrong when keystrokes go missing,
and macOS is the worst case. `macOptionIsMeta` and `attachCustomKeyEventHandler` cover more than
expected, but Cmd chords and IME composition are hand work with less prior art than Electron has.
Contained to one pane type, but do not schedule it as an afternoon.

**Risk — webview inconsistency.** Tauri ships no browser, so the UI runs on WKWebView, WebView2, or
WebKitGTK depending on platform, and they differ. This is the genuine cost of choosing Tauri over
Electron — not runtime access, which the terminal model makes irrelevant. WebKitGTK is historically the
weakest of the three; if CodeMirror or `solid-flow` misbehave there, the fallback is shipping the
Linux build against a pinned newer WebKitGTK rather than changing framework.

**Risk — `solid-flow` maturity.** It carries the whole authoring surface and rates Medium, not High,
on Context7. Spike 3 exists to find out before M4 commits. Fallback if it does not hold: render the
canvas directly with Solid over `@dagrejs/dagre` for layout — more work, no dependency risk.

**Risk — nested agents multiply containers.** An agent built from three agents is four containers, and
each of those could nest further. The cycle check, depth limit, and fan-out cap are not polish;
without all three, one bad graph exhausts the machine.

**Risk — Postgres is now irreplaceable.** With board, wiki, memory, and mail living only in Postgres,
losing the volume loses work that no reindex can rebuild. `locus-postgres` needs a backup command and
a restore drill from M1, not later.

**Cost — scope.** Nine milestones is multi-year at this scope. The phase boundaries are drawn so that
M1 alone is a usable multi-agent runner without any of M2–M8, and M4 is where it stops being a runner
and becomes the product.
