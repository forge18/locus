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
reviewable, and readable by agents working in the repo, not a throwaway planning artifact. `docs/` and
`docs/adr/` in M0 expand it; they do not replace it. `.gitignore` already excludes the generated agent
directories, so `PLAN.md` is tracked normally.

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
| Debug | A **DAP client in Rust**, because agents need to debug. Not VS Code's debug UI. |
| Harness I/O | **ACP first, adapter fallback.** Locus is an Agent Client Protocol client; harnesses that do not speak ACP get a hand-written TOML adapter. Structured output is required either way — no PTY for agents, ever. |
| `local-dx` relationship | Inspiration, not dependency. Locus owns its own registry and schema. |
| Sandbox | One container per agent run. The workspace is a **clone from a local bare remote**, not a mount. Credential handling must be easy and secure; the mechanism is Spike 1's to settle. |
| Kanban + wiki store | Both in Postgres, including wiki revisions. |
| GitHub | Version control, CI/CD, and PRs only. Never Issues or Projects. |
| Marketplace | Git-backed manifest index of CLIs. No MCP servers, ever. |
| Chat | A designated harness session, not a provider API layer. Locus holds no model API keys. |
| Shared services | **Memory, communication, and every other cross-cutting agent capability are implemented in Rust, in `locus-core`, once. Every harness uses that one implementation.** No per-harness variants, no shell-script services, no external daemons. |
| Agents | **Markdown plus a tool list.** Frontmatter (harness, model tier, tools, skills, rules, memory scope) over a prose body. No canvas, no compile — this is what every harness already reads. |
| Workflows | **A visual canvas** (`solid-flow`). A workflow is a **loop toward a goal**; the **goal is the approval gate**; guardrails bound it; `verify` is required and the token budget is optional. |
| Teams | **The workflow is the team.** Its agent nodes are the roster, each node carries a role, and the edges are the dependencies. No separate `Team` entity. |
| Review surface | **Artifacts**, not transcripts. Plans, diffs, diagrams, screenshots, recordings, and a walkthrough on completion — all commentable, and a comment steers the agent that made it. |
| UI components | **Kobalte** headless primitives + **shadcn-solid** components copied into the repo + **Tailwind**. Headless, because an IDE's chrome is small and its large surfaces are all bespoke or bring their own DOM. |

### One clarification carried into the design

The request says both "multi-agent run through terminals, no TUI allowed" and "terminal for coding,
multiple terminals in a UI". These are two different things, and the plan names them separately:

- **Agent Pane** — a typed event stream from one harness. No PTY, no terminal emulator, no TTY
  allocated on the container. Minimizes to a tile showing status and current task. One agent, always.
- **Shell Pane** — a real PTY into a container or the host, for the human. Ordinary terminal.

"No TUI" is therefore structural, not a convention: there is no code path that could render one in an
Agent Pane.

---

## Architecture

### Process topology

```
Tauri application  (locus)
│
├── webview — SolidJS
│     Agent Panes · Shell Panes · Editor · Board · Wiki · Chat · Dashboard
│                            │ typed IPC (tauri::ipc, serde)
└── Rust core — locusd  (in-process; also runnable headless for cron/CI)
      ├── harness registry     load harnesses/*.toml, select adapter by stream format
      ├── run supervisor       spawn, stream, normalize, persist, cancel
      ├── container supervisor bollard → Docker Engine API
      ├── repo manager         local bare remote, per-run clones, branch merge-back
      ├── credential broker   keeps secrets out of containers; network policy, outbound audit
      ├── LSP supervisor       host-side language servers, multiplexed to editor panes
      ├── store                Postgres (sqlx), the single source of truth
      ├── event bus            in-process broadcast + Postgres LISTEN/NOTIFY across processes
      ├── shared services      memory · mail · board · wiki · telemetry · tools
      │                        one Rust implementation, identical for every harness
      ├── workflow engine      loop execution, guardrails, schedules
      └── agent socket         /run/locus.sock — bind-mounted into every agent container
```

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
| `locus-agent-<run_id>` | per agent run | One harness, headless, no TTY. |
| `locus-svc-<project>-<name>` | per project | Services the *project* needs — its own Postgres, Redis, etc., declared in `locus.toml`. |

Network `locus-<project>` joins a project's agents and service containers. Agents reach each other and
the project's services; they do not reach other projects.

Mounts into an agent container:

| Path | Mode | Contents |
| --- | --- | --- |
| `/run/locus.sock` | rw | Host daemon socket |
| `/locus/config` | ro | Harness config materialized for this run |

Plus a `$LOCUS_PORT` unique to the run, and per-project setup/run/teardown scripts from `locus.toml`.

**The workspace is not mounted.** `/workspace` is a *clone* on the container's own filesystem — see
the git model below. **No long-lived credential lives in the container** — see Credentials.

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

- **Overlap is detected at merge, not at claim.** With independent clones, two agents *can* both edit
  the same file — the filesystem cannot stop them the way a shared worktree could. So the claims
  registry becomes **advisory**: it prevents overlapping work from being *assigned*, and the board
  refuses to hand two agents tasks over the same paths, but the hard failure surfaces as a merge
  conflict. Tiered conflict handling matters more here, not less.
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

An agent definition, skill, rule, or command is authored **once**, in Locus. At run start the core
reads `harnesses/<name>.toml` and writes those definitions into the paths and file formats that
harness expects, into `/locus/config` for that run only.

This inverts `local-dx`'s hardest problem. There is no propagation, no symlink graph, no `--prune`,
no drift: the target filesystem is destroyed when the run ends.

Three rules carried over verbatim from `local-dx`, because each earned its place:

1. **Nothing in core names a harness.** Adding one is a TOML file, plus a stream-format module only
   if its wire format is new.
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
  board/       tasks, transitions, claims
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

### Harness contract — ACP first

**Locus is an [Agent Client Protocol](https://agentclientprotocol.com) client.** ACP is JSON-RPC 2.0
over stdio — LSP-for-agents — created by Zed in August 2025 and co-maintained with JetBrains. It is
precisely the layer this design was otherwise going to hand-roll.

- Rust crates `agent-client-protocol` and `agent-client-protocol-schema` are on crates.io and current
  as of mid-2026, giving the wire model natively in the language the core is written in.
- Agents already speaking it: **Claude Agent** (Zed's SDK adapter), **Codex CLI**, **Cline**,
  **Cursor**, **Gemini CLI**, **Goose**, **Factory Droid**, Docker's **cagent**, **GitHub Copilot**
  (public preview), and a growing roster.
- Clients: Zed and JetBrains natively; Neovim and Emacs via community plugins.

**This does not breach the no-MCP rule.** ACP *re-uses MCP types* — JSON type shapes, so integrators
do not invent another representation of a tool call. That is a schema decision, not a server. ACP also
lets a client offer MCP servers to an agent; Locus simply never does.

Two paths, one vocabulary:

| Path | When | Cost |
| --- | --- | --- |
| **ACP client** | The harness speaks ACP | None per harness — it just works |
| **Native adapter** (`harnesses/<name>.toml` + a stream module) | It does not | Written and maintained by us |

Both normalize into the same canonical event set below, so nothing downstream — telemetry, panes,
canvas overlay — knows which path a run came through.

Watch: ACP's remote-agent support is explicitly still in progress. Locus runs agents in local
containers, so the stdio path is the one that matters, but do not build on the remote half yet.

### Native adapter contract

```toml
# harnesses/claude.toml
name    = "claude"
binary  = "claude"
detect  = ["--version"]

[invoke]
argv    = ["-p", "{prompt}", "--output-format", "stream-json", "--verbose"]
stream  = "anthropic-stream-json"     # selects the adapter module
tty     = false                       # never true; the schema rejects true

[events]                              # native name → canonical vocabulary
system            = "session_start"
assistant         = "assistant"
tool_use          = "tool_call"
tool_result       = "tool_result"
result            = "session_end"

[layout]                              # where THIS harness reads things, in-container
agents        = { dir = "/locus/config/agents",  format = "markdown+frontmatter" }
skills        = { dir = "/locus/config/skills",  format = "markdown+frontmatter" }
rules         = { dir = "/locus/config/rules",   format = "markdown+frontmatter" }
commands      = { dir = "/locus/config/commands", format = "markdown+frontmatter" }
context       = { file = "/locus/config/AGENTS.md" }

[capabilities]
resume = true
subagents = true
tool_permissions = true

[model_routing]                       # null = tier absent; fall back UP, never down
low = "haiku" ; medium = "sonnet" ; high = "opus" ; xhigh = "opus-max"
```

**Canonical event vocabulary** — every adapter normalizes to exactly this set:

```
session_start  user  assistant  thinking  tool_call  tool_result  tool_error
permission_request  subagent_start  subagent_stop  aborted  session_end
```

An adapter that cannot produce `session_start` and `session_end` is rejected at registry load, not at
run time.

**Consequence:** a harness that speaks neither ACP nor a headless structured mode cannot be used at
all — not degraded, not PTY-fallback, unsupported. This is the deliberate price of complete, uniform
telemetry and of making "no TUI" structural. ACP shrinks the set this excludes considerably.

**The SDK-only escape hatch.** If a harness ever ships only a TypeScript or Python SDK and no headless
CLI, its adapter is a thin shim that drives that SDK and prints the canonical event stream — and the
shim runs **inside that agent's container**, where a Node or Python runtime already belongs. The host
application never gains a language runtime it did not have.

This is why Tauri costs nothing here. The common argument for Electron is that an in-process Node
runtime is free when the IDE drives agents through TypeScript SDKs. Locus drives agents through
container lifecycles, process streams, Postgres, and git — all first-class in Rust, none better in
Node — and the shared services are mandated Rust regardless, so Electron would mean Electron *plus* a
Rust sidecar and an IPC boundary to own. **Tauri's real cost is webview inconsistency** (WKWebView /
WebView2 / WebKitGTK) rather than runtime access; that is the thing to watch, and M2 should exercise
CodeMirror on all three before the editor work is called done.

### Data model — Postgres schemas

| Schema | Holds |
| --- | --- |
| `core` | projects, repos (multi-repo per project), local remotes, settings |
| `agents` | `agent_defs` (versioned; frontmatter JSONB + Markdown body), sessions, runs, parent/child run edges, normalized events, **artifacts and their comment threads** |
| `board` | boards, columns, tasks, **dependency edges**, transitions, assignments, task↔run links, evidence links |
| `wiki` | pages (**typed by kind**), revisions, links, contradictions, ingest log, embeddings (`pgvector`) |
| `memory` | **core** (bounded, per-agent and per-project, always injected) and **store** (facts, scope, provenance, embeddings, confidence, decay) |
| `workflows` | `workflow_defs` (versioned; `graph` + `spec` JSONB), schedules, executions, iterations, guardrail trips, verify results |
| `mail` | threads, messages, delivery state |
| `market` | manifests, installs, per-image tool sets |

### What a session is

The word is overloaded — ACP has sessions, every harness has sessions, and the UI has panes. Locus
needs one definition, and this is it:

```
Project
└── Session          a durable, named thread of work with ONE agent
    ├── Run          one container lifetime = one ACP session
    │    └── Turn    one prompt → one response
    ├── Run          (after a loop reset: new container, same session)
    └── Run
```

| | **Session** | **Run** |
| --- | --- | --- |
| Bounded by | you closing it | the container exiting |
| Holds | agent@version, its branch on the local remote, the board task it serves, core-memory base, pane state | events, token usage, exit status, artifacts |
| Resumable | yes — by starting another run | no; a run is over when it is over |
| Maps to | the harness's own conversation id, where it has one | one ACP session instance |
| Cost | the sum of its runs | measured directly |

**The session is what survives the reset.** That is the whole reason for the split. The Ralph-loop
pattern the field converged on — pick, act, validate, commit, *reset the context* — needs something
that persists across resets and something that does not. The run is the thing that resets; the session
is the thing that accumulates. A workflow iteration ends a run and starts a new one **in the same
session**, and memory, branch, and task linkage carry across because they belong to the session.

Three consequences worth stating:

- **A Locus session is not an ACP session.** An ACP session maps to a *run*. Where a harness supports
  resume, the core stores its native session id on the run and hands it back on the next one.
- **A Shell Pane is not a session.** It is a PTY attached to a container or the host. No agent, no
  events, no cost attribution — deliberately a different thing.
- **Chat is a session**, with the designated spec agent. Nothing special about it.

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
screenshot; the comment routes back into the session that produced it and the agent responds. It is
the PR review interaction, applied to plans and images rather than only code — and it is the same
mechanism as the agent-authored PR flow in M7, so it is one implementation, not two.

Artifacts are rows attached to a session, and they arrive in the inbox when they need you.

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
That was wrong once agents need to debug: the core needs a **DAP client** regardless. What it does not
need is VS Code's debug *UI* — and a client is a fraction of the cost of an extension host. A human
debug pane, if ever wanted, is then a view over a client that already exists.

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

**Contradiction flags at ingest time, not query time.** This is the idea most worth stealing. When a
new source contradicts an existing claim, the conflict is raised *when it lands* — as a row in
`wiki.contradictions` and a card on the board — rather than discovered months later by whoever
happened to read both pages. The same detection serves memory: a store-tier fact that conflicts with a
wiki claim is the same problem.

**A wiki linter.** `locus wiki lint` reports orphan pages, broken links, entities mentioned but
never given a page, and gaps where a claim has no source. This is the same discipline `dx-lint`
already applies to code, pointed at knowledge.

**A graph view, nearly free.** Pages are nodes, `[[wikilinks]]` are edges. `solid-flow` is already in
the app for the Workflow Canvas, so rendering this costs a palette, not a subsystem.

### Knowledge, as one model

Three layers, and it matters that they are distinct:

| Layer | Bounded? | Written by | Read by |
| --- | --- | --- | --- |
| **Core memory** | Yes — hard cap | The agent, under pressure to consolidate | Always in the prompt |
| **Store memory** | No | Agents, provenance-tracked | `locus memory recall` |
| **Wiki** | No | Ingest, then curated | `locus wiki search/read`, and humans |

Core memory is what an agent *is*. Store memory is what it can *look up*. The wiki is what the
*project* knows, in prose a human would want to read. Collapsing any two of them is how you get either
a bloated prompt or a knowledge base nobody trusts.

### Memory — two tiers, borrowed from Hermes

Nous Research's Hermes Agent takes the opposite position to almost everyone else: memory is not
retrieved, it is **bounded, curated, and always in the prompt**. `MEMORY.md` is capped at 2,200
characters and `USER.md` at 1,375. Their argument is worth quoting because it is the correct one:

> Context has no curation. It's a dump… Memory is curated. It's the distillation of experience into
> something compact and actionable. It doesn't grow indefinitely — it consolidates, updates, and
> forgets.

Hermes also stacks *one optional external provider* beside those files for retrieval. **Locus is that
external provider — for every harness at once.** So this is not a competing model, it is the other
half of one, and Locus should implement both halves:

| Tier | What | Mechanism |
| --- | --- | --- |
| **Core** — always in the prompt | The agent's own bounded notes, plus a profile of the user and project | Hard character cap. **A write over the cap returns an error**; nothing is auto-compacted. The agent must consolidate or forget in the same turn before retrying |
| **Store** — recalled on demand | Everything else: scoped, provenance-carrying, cross-agent, cross-harness | `locus memory recall`, hybrid retrieval, prefetch before a turn and sync after |

Four mechanisms taken directly from Hermes, each solving something the obvious design gets wrong:

1. **Refuse, don't auto-compact.** Silent eviction lets memory rot invisibly. An error forces the
   agent to decide what to forget, which is the only moment it has the context to decide well.
2. **Freeze the core tier at run start.** Writes persist to Postgres immediately but do not change the
   injected block mid-run — mutating the system prompt destroys the model's prefix cache. Tool
   responses show live state, so the agent is never lied to.
3. **No read tool for the core tier.** It is already in the prompt; a `recall` call against it would
   be pure waste. Only the store tier answers queries.
4. **Separate agent notes from user/project profile.** Different owners, different lifetimes, and
   mixing them is how both get stale.

Hermes ships one more warning that reads as a direct description of what Locus is building:

> Don't point two agent processes at the same Hermes home directory… two writers sharing one home will
> compound each other's entries into state neither of them (nor you) authored.

They solved it by forbidding it. **Locus solves it instead** — that is what scoping, provenance, and
Postgres's MVCC with per-run advisory locks are for. A shared memory that several agents write
concurrently is precisely the thing the field has not built, and it is why the store is a database and
not a file.

**Store-tier recall is hybrid, not embeddings-only.** Structured filters (scope, agent, recency,
confidence) then `tsvector` then `pgvector`, in that order of selectivity. Gastown's "beads" pattern
makes the case: immutable, provenance-carrying decision records that are *SQL-addressable* beat vector
RAG for institutional memory, because the question is usually "what did we decide about X, and who
decided it" rather than "what text is semantically near X".

**Provenance on every row** — which agent wrote it, in which run, from which source. That is what lets
one agent's recall be filtered out of another's, and a wrong memory be traced back and deleted rather
than merely contradicted.

**One boundary the two sources disagree about, resolved.** Hermes lets an agent write its own core
memory freely. Gloaguen et al. (ETH Zurich) measured LLM-generated `AGENTS.md` files as giving no
benefit — roughly 3% *worse* on average, with 20%+ higher inference cost — while human-written ones
gave about 4% improvement. These are not in conflict, because they describe different files:

- **Agent-scoped core memory** — what *this* agent learned about its environment. Agent-writable,
  under the cap. Hermes is right.
- **Project context loaded for every agent** — the `AGENTS.md` equivalent. **Human-written and
  human-gated.** The ETH finding is right. An agent proposes via a `REFLECTION` record after a run —
  what surprised me, one pattern worth keeping, one prompt improvement — and a person accepts or
  discards it.

Conflating those two is how a project's instructions fill with plausible machine-written noise.

**`board` carries dependency edges.** A task can be `blocked_by` others; completing one flips its
dependents from `blocked` to `pending` automatically, so a waiting agent picks up work the moment it
is unblocked rather than when a human notices. This is the mechanism Claude Code's Agent Teams uses,
and it is the difference between parallel execution and coordinated execution. When a workflow creates
tasks, these edges are **generated from the workflow graph** — dependencies are declared once, on the
canvas, not re-entered on the board.

**`board` also carries evidence links.** Every transition an agent makes cites the run and the specific
events that justify it — terminal output, test results, the verify command's exit. Codex Web calls
this verifiable evidence; the telemetry already stores it, so this is a join, not new capture.

Postgres rather than SQLite for the reason `local-dx` measured: SQLite permits one writer, so
concurrent agents either collide on `SQLITE_BUSY` or queue. Carry forward its two hard-won fixes —
scrub NUL bytes at the insert boundary, and take a transaction-scoped advisory lock keyed on the run
so only genuinely colliding writers serialize.

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
model_tier: high        # low | medium | high | xhigh — resolved per harness
tools: [rg, gh, cargo]  # allowlist, resolved against the marketplace index
skills: [audit-code]
rules: [no-secrets]
memory: { scope: project, write: propose }
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
  call `locus agent invoke` at runtime if that is in its tool list — bounded by a hard depth cap and
  fan-out cap in the core — but *authored* composition is what the Workflow Canvas is for.

Agent definitions live in Postgres like everything else, with import and export as `.md` so they can
be reviewed in a PR or copied between machines when that is what you want.

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
| `Agent` | an agent definition, pinned by version, **plus a `role`** — the definition is reusable, the role is per-workflow |
| `Task` | a unit of work, optionally sourced from the board |
| `Loop` | the iteration construct: what repeats, and what resets between passes |
| `Condition` | deterministic routing. **No model in the orchestration path** — the graph decides, not an LLM |
| `Gate` | a checkpoint: human, or another agent acting as reviewer |
| `Verify` | the runnable success criterion. Required |
| `Guardrails` | the limits below, attached to the workflow |

**Workflow guardrails.** Each is borrowed from a measured failure in an existing tool. Defaults apply
to any agent run; a workflow may tighten or relax them, and they are what make leaving a loop
unattended defensible:

| Guardrail | Default | Why |
| --- | --- | --- |
| `max_iterations` | 8 | Agents loop endlessly retrying the same broken approach without a hard stop |
| Forced reflection before retry | on | "What failed? What specific change would fix it? Am I repeating myself?" — reported to substantially cut stuck agents |
| Kill-and-reassign after 3 stuck iterations | on | A fresh context beats a confused one |
| Idle detection | 60s | Agent Orchestrator's measured flaw was agents idling 90+s on tool approval with nobody told. Idle shows on the tile and notifies |
| Wall-clock ceiling | none | Optional; a loop that cannot finish overnight should stop, not run to morning |
| **Token budget** | **none — optional** | When set, auto-pause at 85% and notify rather than draining silently. Nested agents multiply spend, so a ceiling is worth having available even if rarely used |

**Reviewer agents need no special machinery.** A reviewer is an ordinary agent with read-only tools,
wired to a `Gate` that triggers on task completion — roughly one reviewer per three or four builders.
The `Gate` and `Verify` nodes exist so the pattern is expressible without hand-wiring.

### Teams — the workflow *is* the team

There is no `Team` entity. A workflow already declares everything a team is:

| A team needs | The workflow supplies |
| --- | --- |
| A roster | its `Agent` nodes |
| A role per member | the `role` field on each `Agent` node |
| Who waits on whom | the edges between them |
| A shared task list | the board scope the workflow owns |
| Peer messaging | `locus mail`, available to every agent regardless |
| File-level conflict prevention | claims with path-overlap rejection, already in M3 |
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
agent_defs ─┬─ frontmatter JSONB      │           loop with no termination
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
per node. The Agent Pane remains the event stream; the canvas is the map. Both read the same
normalized events, so there is one source and two renderings.

### Agent CLI (`locus`, inside the container)

```
locus memory note add|replace|remove          the bounded core tier; over-cap writes ERROR, never evict
locus memory recall <query>                   the store tier: structured + FTS + vector
locus memory write --scope … --why …          a fact, with provenance attached automatically
locus memory forget <id>                      traced back to its writer, then deleted
locus mail send|list|read|reply|drain|wait   agent↔agent, Rust-native
locus task list|show|move|claim|comment      board, agent-driven
locus wiki search|read|write|history         wiki, with revisions attributed to the run
locus wiki ingest <path|url>                 read a document into typed pages; flags contradictions
locus wiki query <question>                  synthesize an answer, filed back as a synthesis page
locus wiki lint                              orphans, broken links, missing entities, unsourced claims
locus lsp def|refs|hover|symbols|rename       semantic navigation over this run's clone
locus debug break|run|step|stack|eval|vars    DAP, against the code as it actually runs
locus browse open|click|fill|assert|screenshot  drive the project browser; shots become artifacts
locus agent invoke <name>@<version>          run a nested agent: own container, own clone
locus svc up|down <name>                     start a project service container; agents get no Docker socket
locus ask <question>                         escalate to the human via the Chat pane; blocks
locus run status|artifacts                   this run's own state
locus artifact put <kind> <path>             publish a plan, diagram, or image for review
locus artifact comments                      feedback left on your artifacts, threaded
locus tools list|docs <name>                 marketplace docs for allowlisted CLIs
```

`--json` on every command, because the caller is usually a model.

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
injects each tool's `docs` into context. A tool not in the allowlist is not installed, so the agent
cannot reach for it.

The marketplace splits across two milestones, because agents need the *index* long before they need
the *installer*: reading manifests to validate an agent's `tools` list and inject docs is cheap and
lands at M4; image baking and install land at M8.

### Frontend and IPC constraints

Three rules that are cheap to follow from the start and expensive to retrofit.

**Channels for streams, events for notifications.** Tauri's own documentation is explicit: the event
system "is not designed for low latency or high throughput situations" and works by evaluating
JavaScript; `tauri::ipc::Channel<T>` is what they use internally for child-process output. Every
high-frequency path here is a Channel:

| Path | Mechanism |
| --- | --- |
| Shell Pane PTY bytes | `Channel<&[u8]>` |
| Agent Pane normalized events, including token deltas | `Channel<Event>` |
| LSP diagnostics and semantic tokens | `Channel<T>` |
| "a run finished", "a task moved", "a claim was refused" | `emit` — low frequency, many listeners |

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
| Shell Pane | xterm.js |
| Board, wiki, dashboard | bespoke |
| Dialogs, context menus, tabs, tooltips, combobox, toast, palette | **Kobalte**, styled via **shadcn-solid** copied into `src/ui/` |
| Long lists | `@tanstack/solid-virtual` |

Kobalte ships Combobox, Context Menu, Tabs, Dialog, Tooltip, and multi-region Toast. It ships **no**
resizable split panes and no tree — correctly, because both of those are the pane manager and the file
tree, which are product here rather than components. shadcn-solid is copied in rather than depended
on, so nothing visual is version-locked, and it sits on Kobalte so there is one primitive layer.

**Keyboard capture is a Shell Pane problem, not an app problem.** Agent Panes render typed events and
need no raw key access; this design has no PTY for agents at all. The Shell Pane is the only true
terminal, and it is where macOS eats keystrokes before JS sees them.

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
- Language servers are spawned and supervised **on the host** by the LSP supervisor, one set per
  project, shared across panes. Agents' containers do not run language servers.

**One editor, two zoom levels.** The side pane beside an agent and the full-window editor module are
the same CodeMirror components at different sizes, sharing one keymap, one theme, one LSP client. A
second editor for the "real" module would mean two of each and muscle memory that breaks whenever you
move between them — and would re-import the whole extension-host cost for the surface you use *least*
in an agent-first IDE. File tree, tabs, splits, find-in-files, go-to-symbol, and multi-cursor are all
either native to CodeMirror or chrome built once and reused at both sizes.

Accepted gaps, restated so they are not rediscovered later: no VS Code extensions, and Lezer grammar
coverage thins out in the tail — Odin and GDScript have none. Mitigation when it bites is LSP semantic
tokens for color plus tree-sitter-WASM decorations for structure.

**No longer a gap: debugging.** Agents need to debug, so a DAP client lands in the core regardless of
editor choice. A human debug pane, if ever wanted, becomes a view over a client that already exists.

---

## Phased roadmap

Each milestone ships a runnable `verify:`.

### M0 — Specification and spikes  ← the immediate deliverable

Write the spec set, then answer the two questions that could invalidate it.

- `docs/architecture.md`, `docs/harness-contract.md`, `docs/data-model.md`, `docs/agent-cli.md`,
  `docs/marketplace.md`, `docs/roadmap.md`
- `docs/knowledge-model.md` — the three layers (core memory, store memory, wiki), what writes each,
  the bounded-core mechanics, and contradiction handling across memory and wiki
- `docs/sandbox-model.md` — credentials, network policy, the git model, and the threat model
- `docs/session-model.md` — session vs run vs turn, and what survives a reset
- `docs/artifacts.md` — the artifact kinds, the walkthrough, and how comments steer
- `docs/agent-capabilities.md` — `locus lsp`, `locus debug`, `locus browse`: where each server runs
  and why the client is shared but the server is not
- `docs/agent-format.md` — the frontmatter schema, the tool allowlist, and how it materializes
- `docs/workflow-canvas.md` — node vocabulary, loop semantics, guardrails, the compile pipeline
- `docs/landscape.md` — what the 2026 field has, what it lacks, and which features were taken from
  where. Keeps the borrowed ideas attributed and the bet legible
- `docs/adr/`, in order:

  | ADR | Records |
  | --- | --- |
  | `0001-codemirror-over-vscodium.md` | The editor choice and the gaps accepted with it |
  | `0002-structured-output-required.md` | Why no PTY for agents, and what that excludes |
  | `0003-container-per-agent.md` | The sandbox boundary and credential injection |
  | `0004-postgres-single-store.md` | One store; what is derived and what is not |
  | `0005-cli-not-mcp.md` | The agent surface is a binary, not a server |
  | `0006-shared-services-in-rust.md` | Memory, mail, and the rest implemented once |
  | `0007-agents-markdown-workflows-canvas.md` | Agents are Markdown + a tool list; only workflows are graphs, and why the split falls there |
  | `0008-tauri-over-electron.md` | The Electron case rests on in-process SDK orchestration, which this design rules out; accepted costs are webview inconsistency and Shell Pane keyboard work |
  | `0009-acp-as-primary-harness-protocol.md` | ACP first, native adapters as fallback — including why re-using MCP *types* is not running MCP *servers* |
  | `0010-human-gated-context-promotion.md` | The ETH Zurich finding, and what agents may not write |
  | `0011-workflow-is-the-team.md` | Why there is no `Team` entity, and the reproducibility that buys |
  | `0012-session-run-turn.md` | The session/run split, and why the session is what survives a reset |
  | `0013-one-editor-two-zoom-levels.md` | Why no second editor for the full module, and that DAP arrives as a client regardless |

- `docs/frontend-constraints.md` — Channels vs events, one webview per window, keyboard capture
- Fill in the stub `AGENTS.md` at the repo root
- **Spike 1** `spikes/01-sandboxed-harness/` — does a harness run in a container, authenticated
  without holding a long-lived secret, against its own clone, emitting a parseable event
  stream? This is the highest-risk unknown in the whole design: if harness auth cannot be injected
  without baking a secret into an image, the sandbox model changes. **Do this over ACP**, so it also
  proves the `agent-client-protocol` crate carries a real session end to end.
- **Spike 2** `spikes/02-editor-embed/` — CodeMirror 6 + `@codemirror/lsp-client` against a real
  `rust-analyzer` inside a Tauri window, plus a `MergeView` over a real git diff.
- **Spike 3** `spikes/03-workflow-canvas/` — `solid-flow` with custom typed nodes, a graph that
  round-trips through JSONB unchanged, and a loop construct whose termination is checkable at compile
  time. Its reputation on Context7 is Medium, not High, so confirm it carries this weight before M4
  depends on it.

`verify:` all three spikes produce a written answer in their directory; `docs/` is internally
consistent.

### M1 — Core runtime

Daemon, store, registry, containers, Agent Panes.

- `locus-postgres` lifecycle, migrations (`sqlx`), and **backup/restore**, since memory and mail will
  live here and cannot be rebuilt
- **ACP client** on the `agent-client-protocol` crate — the primary path
- Harness registry, TOML schema validation, **one** native adapter to prove the fallback path exists
- **Agent definitions**: Markdown + frontmatter, versioned, materialized per run. Because agents are
  not a graph, they land here rather than waiting for M4 — the product is usable four milestones
  earlier than the previous draft implied
- Container supervisor (`bollard`), image build, mount and credential injection
- **Session and run model**: sessions persist across runs, runs bounded by the container
- **Artifacts**: plan, diff, and walkthrough kinds, attached to sessions, with comment threads that
  route feedback back into the session. Media kinds arrive with the browser at M3.5
- Run supervisor: spawn → stream → normalize → persist → cancel
- Credential broker and egress policy, whichever shape Spike 1 settles on
- Tauri + SolidJS shell: JS-side pane manager over **one webview per window**, Agent Pane rendering
  typed events, minimize-to-tile
- Shell Pane with a real PTY (xterm.js), clearly a separate pane type — and the keyboard work lives
  here and nowhere else
- All streaming over `tauri::ipc::Channel`, coalesced per pane on a frame tick, from the first commit

`verify:` two agents run concurrently in separate containers against the same repo — **one over ACP,
one through a native adapter** — and their events are indistinguishable downstream; every event lands
in Postgres; both minimize to informative tiles. A Shell Pane survives a `vim` session with
Option-as-Meta, Cmd chords reaching the app rather than the menu bar, and IME composition intact.

### M2 — Workspace

- File tree, CodeMirror editor pane, side-pane and full-window modes — **the same components at two
  zoom levels**
- LSP supervisor and multiplexing, host-side, serving the editor panes
- **`locus lsp` for agents** — the same Rust client, but running in the agent's container against that
  run's clone. Lands here because it shares the client with the editor's
- `MergeView` diff review over an agent's pushed branch
- Search across the project
- Exercise the editor on **all three webviews** — WKWebView, WebView2, WebKitGTK. This is Tauri's
  real tax and the only place it lands hard

`verify:` open a file, get real completions and diagnostics, review an agent's diff, accept a hunk —
on each of the three webview engines. Separately, an agent in a container resolves a symbol's
definition through `locus lsp` and gets the same answer the editor does.

### M3 — Coordination, memory, and mail

The shared-services milestone. Everything here is Rust in `locus-core`, surfaced identically to every
harness through the `locus` CLI.

- **Memory, both tiers**: the bounded core (hard cap, refuse-don't-evict, frozen at run start to
  preserve the prefix cache) and the store (scoped, provenance, hybrid recall, decay)
- **Mail**: threads, delivery, `wait`/`drain` semantics, `locus mail` verbs
- Repo manager: bare local remote per project, per-run clones with `--reference` against a shared object store
- Claims registry with path-overlap rejection across live claims — the mechanism `fleet` already
  proved at `~/Repos/fleet`
- **Agents push to a local git remote; you `git fetch && git checkout` from your normal repo.**
  Sculptor's pattern, and better than bind-mounting a workspace into the human's editor: it
  keeps Locus out of your editor, your merge tool, and your shell, and it means reviewing an agent's
  work uses the git you already know
- Merge-back with tiered conflict handling
- Run guardrails: `max_iterations`, forced reflection before retry, token budget with auto-pause, kill
  and reassign when stuck, idle detection surfaced on the tile

`verify:` three agents on overlapping work; overlap is refused at claim time; two exchange mail; one
writes a memory that a **different harness** later recalls; all three merge back or report a
structured conflict.

### M3.5 — Agent capabilities: debug and browser

Small, and it belongs before the canvas because a workflow that cannot verify a UI or inspect a stack
is a workflow that escalates to you constantly.

- **DAP client in Rust** → `locus debug`. Adapters run in the agent's container
- **Browser container per project** → `locus browse`. Playwright driving Chromium on the project
  network, reaching the agent's app on `$LOCUS_PORT`
- **Image and recording artifacts** from `locus browse`, attached to the session and the board card,
  so a UI change is reviewable without checking out the branch

`verify:` an agent sets a breakpoint, inspects a variable, and fixes a bug it could not have found
from print statements; a second agent changes a page, records the flow, and the recording appears as a
commentable artifact — and a comment left on it reaches the agent.

### M4 — Workflow Canvas

Agents already work — they are Markdown and shipped in M1. This is where *orchestration* becomes
authorable, and where the product's character arrives.

- `solid-flow` canvas, the workflow node types, typed handles, graph validation
- Compile pipeline: `graph` JSONB → `spec` JSONB → versioned `workflow_defs`, round-tripping exactly
- **Loop execution**: pick → act → validate → commit → reset, with memory carrying across resets
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
authored. A loop with no termination condition is refused at compile time, not at run time.

### M5 — Project management

- Board: projects, columns, tasks, drag, agent-driven transitions
- **Dependency edges with auto-unblock** — completing a task flips its dependents to `pending` and a
  waiting agent picks them up without a human noticing first. Edges are generated from the workflow
  graph that created the tasks
- **Evidence links** on every agent-made transition, citing the run and the events that justify it
- Wiki: **typed pages** (source, entity, concept, synthesis, decision, overview), **ingest** via
  `markitdown`, **contradiction flags raised at ingest time** as board cards, `locus wiki lint`, a
  `solid-flow` graph view over `[[wikilinks]]`, GUI editor, revisions, `pgvector` search
- Chat pane as a designated harness session that can draft tasks and answer questions
- **Reflection review queue** — the human gate where an agent's proposed learnings are accepted into
  always-on project context or discarded

`verify:` an agent moves a task to Done and cites the run and the test output that justifies it; a
blocked task auto-unblocks and is claimed without human input; a wiki page edited in the GUI is read
back by an agent in a container; ingesting two documents that disagree produces a contradiction card
rather than two quietly conflicting pages.

### M6 — Automation and discoverability

- Schedules: cron → workflow, with executions recorded against their verify result
- Dashboard: runs, spend, verify pass rate, guardrail trips, board throughput, agent trust
- Discoverability: command palette, global search across code, wiki, tasks, and run history

`verify:` a scheduled workflow runs unattended overnight and reports green from its verify command;
a second one trips its token budget, auto-pauses at 85%, and notifies rather than draining.

### M7 — GitHub

Version control, CI/CD, and PRs only.

- `gh` + `gix` in core: branch, PR open/review/merge, CI status
- CI status surfaced on the board and dashboard
- **CI babysitter** — a failing pipeline pulls its logs, feeds them to an agent, retries a bounded
  number of times, then escalates. Sculptor and Agent Orchestrator both ship this and it is the single
  most-cited reason people leave a run unattended

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

- `docs/architecture.md` and the ADR set under `docs/adr/` — M0's whole output
- `harnesses/*.toml` — one per harness; the file that keeps harness names out of core
- `crates/locus-core/` — registry, adapters, supervisors, store, **and every shared service:
  `memory/`, `mail/`, `board/`, `wiki/`, `telemetry/`, `tools/`**
- `crates/locus-cli/` — the in-container agent CLI; a thin socket client with no logic of its own
- `apps/desktop/` — Tauri + SolidJS. `workflow-canvas/` (node components, compile client, overlay),
  `panes/` (the pane manager), `ui/` (shadcn-solid components copied in, owned here)
- `migrations/` — the eight schemas
- `spikes/01-sandboxed-harness/`, `spikes/02-editor-embed/`, `spikes/03-workflow-canvas/`

Reuse rather than rebuild:

- **`agent-client-protocol`** (crates.io) — the ACP wire model in Rust. Replaces most of what the
  harness registry was going to be.
- **Vibe Kanban** (BloopAI, now community-maintained after bloop shut down in April 2026) — the
  closest prior art for board-drives-agents UX, open source and free to read.
- **Sculptor** (Imbue, MIT) — container-per-agent and the local-git-remote handoff pattern.

- `~/Repos/fleet` — claims registry, path-overlap rejection, tiered merge-back conflict handling, and
  its overlap doctrine. Claude-Code-specific and worktree-based as written, but the conflict-tier
  algorithms port to merge-time handling.
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

M0 is complete when:

```bash
# the spec set is internally consistent and every ADR states a decision, not a survey
ls docs/*.md docs/adr/*.md

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

**Risk — adapter-only excludes harnesses.** Some harness will speak neither ACP nor a headless
structured mode. The decision is deliberate and ACP shrinks the excluded set considerably, but the
exclusion is real and will be felt.

**Risk — ACP evolves under us.** Taking a dependency on someone else's protocol trades per-harness
maintenance for protocol-churn maintenance. The mitigation is that the native adapter path stays
built and tested from M1, so ACP is never the only way in. Its remote-agent half is explicitly
unfinished; do not build on it.

**Risk — no debugger.** CodeMirror buys a small, Solid-native surface at the cost of DAP. If
step-debugging turns out to matter, the fix is a separate debug pane speaking DAP directly, not a
change of editor.

**Risk — Shell Pane keyboard fidelity.** A webview terminal feels wrong when keystrokes go missing,
and macOS is the worst case. `macOptionIsMeta` and `attachCustomKeyEventHandler` cover more than
expected, but Cmd chords and IME composition are hand work with less prior art than Electron has.
Contained to one pane type, but do not schedule it as an afternoon.

**Risk — webview inconsistency.** Tauri ships no browser, so the UI runs on WKWebView, WebView2, or
WebKitGTK depending on platform, and they differ. This is the genuine cost of choosing Tauri over
Electron — not runtime access, which adapter-only makes irrelevant. WebKitGTK is historically the
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
