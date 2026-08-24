# UI mockup review — 2026-08-23

A full screen-by-screen read of the current desktop mockup against `.specs/`, `PLAN.md`, and the
code in `crates/locus-core` and `apps/desktop`. Twenty-nine views plus the agent panel.

**Verdict: the spec layer describes the wrong mockup.** M0.6 (`design-desktop` + five `desktop-*`
slices, 195 tasks, closed) was written from an earlier iteration. The current design renames four
rail categories, cuts the plan pipeline from nine stages to seven, and adds six surfaces that no
spec covers. `locus-core` is broadly ahead of the frontend and mostly matches the *old* contract.

## Which file is authoritative

| File | Status |
| --- | --- |
| `docs/UI mockups for PLAN.md/Locus v2.dc.html` | **Current.** 6,622 lines, 29 views. |
| `docs/UI mockups for PLAN.md/AgentPanel.dc.html` | **Current.** The agent panel, embedded by Interact and by Plan → Converse. |
| `docs/UI mockups for PLAN.md/Locus UI mockups.html` | Byte-identical bundle of the current file, fonts inlined. No extra artboards. |
| `docs/UI mockups for PLAN.md/design_handoff_locus_v2/` | **Superseded.** Its README describes the old navigation and the nine-stage plan; M0.6 was written from it. |
| `docs/UI mockups for PLAN.md/Locus.dc.html` | v1, 14 views. Historical. |

The current mockup ships no README. Until this document existed, the superseded README was the only
prose description of the design, and it reads as current — the failure mode that produced this
review.

## Decisions recorded here

1. **The plan pipeline is seven stages.** Audit and Override are removed as stages. The auditor
   remains an agent role that runs on a schedule; user override is expressed by editing during
   Recommend and Decompose. Core currently ships nine (`crates/locus-core/src/services/planning.rs`).
2. **ACP-only stands.** The Telemetry screen shows a "capture source" facet with four values
   (hooks, acp, stream-json, session-log). `.specs/acp-client` and `.specs/telemetry` settled on ACP
   as the only transport. The facet is stale fixture content and is not implemented.
3. **The registry decides the harness roster.** The mockup lists `hermes` and hardcodes "12
   harnesses" in the materialization sidebar. The registry loads eleven and rejects `hermes` as
   ACP-incapable. Mockup rosters are illustrative.
4. **Unsigned uploaded CLI tools stay rejected.** The mockup offers unsigned tools to read-only
   roles, but their install path still executes during image construction. `PLAN.md` keeps the
   fail-closed Minisign gate: an invalid, missing, or untrusted signature never enters the catalog
   or an image; the UI offers signing instead of a role exception.

---

## Navigation

**Title bar**, 42px: traffic lights, `LOCUS`, current category and view label, then two pills.

- **Dispatch pill** — broadcast icon, running count, pulsing dot. Opens an activity popover with
  two tabs, *Attention needed* and *All*. Rows carry icon, title, elapsed, project tag, meta.
  Footer: **Stop all** (destructive) and **Open Dispatch**. Tab copy differs — attention: "Three
  runs are blocked on you. Everything else is running and does not need a decision."; all: "Runs
  first, then what has already happened. Nothing here is an obligation unless it is in the first
  group."
- **Inbox pill** — tray icon, count badge. Opens a quick-preview popover listing items that need a
  response, footer **Open Inbox**.

**Rail**, 212px, two groups.

- **Project** — a switcher pill (`#project`, type-to-filter with match highlighting, per-project
  running/spend note, and a **+ New project** row) followed by **Setup · Plan · Manage · Interact ·
  Review**.
- **Cross-Project** — **Analytics · Memory · Settings · Workshop**. Memory expands to Short-term,
  Long-term, Artifacts, Wiki. Workshop expands to Agents, CLI, Commands, Harnesses, Hooks, Linters,
  Output styles, Providers, Rules, Skills, Workflows.

Renames against the superseded design: Develop → **Interact**, Automate → **Manage**, Dashboard →
**Analytics**, Projects → **Setup**. Inbox and Dashboard leave the rail entirely. Review's landing
view changes from Telemetry to a new QA screen.

Every view has a `locus://` locator (`locus://<project|all|app>/<kind>/<id>`). A category click
lands on that category's first view.

**⌘K locator palette** — sections *Needs you*, *Running now*, *Where you were*; each row carries its
locator. Footer: "Opens on a list — recognition, not recall." Keys: `↑↓ move · ↵ open · ⇧↵ scope ·
esc close`.

**Toasts** — bottom right, dismissible, suppressed on Interact and while the Dispatch popover is
open. Footer: "Nothing here needs an answer. Things that do go to your inbox."

**Merge modal** — shared overlay, "The work becomes yours". Names the branch, the commit split, and
"the only irreversible step in the run". Two columns: *Evidence travelling with it* (verify command
and exit code, plan clauses satisfied, analyzer result, and how many changed files you actually
opened) and *Size of the change* (files, added, removed, plus whether it is inside the guardrail).
A warning box names files you did not open: "an approval granted without opening the artifact is the
measured failure mode, not a hypothetical one." Buttons: **Merge and close the task**, **Open the
two files first**. Recorded as a review, not a steer.

---

## Screens

## Inbox

Two panes. Left: **To do** / **Completed** tabs, a throughput strip (`3 / 6 per hour`, "under
budget"), and a per-view project filter ("Filters this list only. Every other screen keeps its
own."). The list is an `aria-live` log.

Item types, each documenting the response it wants: **Gate**, **locus ask**, **Guardrail**. Footer:
"Every item type documents the response it wants. An item with no response is a notification, and
notifications go to Activity."

**Completed** keeps resolved items grouped by day with time-to-resolve — "Kept so the resolution is
auditable — what you decided, and how long a loop waited on you for it."

Detail pane for a Gate: tag, title, locator, agent and role, gate mode (`human`). Renders the plan
under review, an info callout naming the irreversible step, a comment box ("Comment steers the agent
that made it"), then **Approve & release the loop** / **Send back with comment**. Two footnotes:
*Why this is here* (which workflow node, set to `human`, and that the agent is blocked, not idle)
and *Cost of waiting* ("One loop held for 4m. No tokens burn while blocked.").

## Analytics (global)

Subtitle: "All projects — the one surface that ignores the project selector."

- Range tabs **24h / 7d / 30d / 90d / All**, each mapping to a bucket count and unit (24 hourly, 7
  daily, 30 daily, 13 weekly, 12 monthly).
- Four stat cards — **Spend**, **Tokens**, **Cache read**, **Runs**. Picking one redraws everything
  below it against that measure. "Pick a card to redraw everything below it against that measure."
- Trend chart with metric tabs **Spend / Tokens / Cache read**.
- Breakdown table, dimension tabs **Model / Harness / Agent / Role / Workflow**, columns
  `<Dim> · Tokens · Cache · Spend · Runs · Per run`. Subtitle: "Same rows every time — the bar
  tracks the measure you picked."
- **Tasks** card: outcome bars **Landed / Abandoned / Still open**; cost-by-role table
  `Role · Landed · Cost · Runs · First try`; a "Most expensive to land" list (iterations, cost).
- **Run times by workflow**: `Workflow · Runs · Median · p90 · Iter · Verified`, with a dual
  median/p90 bar. "Median and p90 wall-clock per run. The gap between them is the tail you feel."
- **Memory retrievals** by tier — Short-term ("already in the window — no fetch"), Long-term ("facts
  recalled by the keeper"), Artifacts ("prior run output re-read"), Wiki ("curated pages pulled
  in") — each with hits, useful %, average tokens. Stat chips: recalls per run, recalls that changed
  the answer, facts written, promoted to long-term. Plus a "Most read" list.
- **Extension usage**: kind filter (all, skill, rule, hook, linter, style, agent) over a list of
  extension name, hit count, and a note ("loaded in 34% of runs", "1 failing").

### Analytics → Telemetry (sub-tab)

Sub-tabs **Overview / Telemetry**. Telemetry is a 264px facet rail plus a query surface.

- Facets, each showing result-set counts: harness, capture source *(see Decision 2 — dropped)*,
  project, agent · role, model tier, verify, arbiter class, branch. The branch facet shows
  `agent/* 641` and `main 0` with the footnote "Locus never works in main — the zero is the
  invariant holding." Rail footer: "Every facet is a column on the normalized event log. Counts are
  the result set, not the corpus."
- BM25 search over the normalized log, plus removable active-filter chips and **Reset filters**.
- Stat cards: Sessions, Events, Tool errors, Output tokens, and a sessions-over-time sparkline.
- **Actions** panel — the canonical verb vocabulary with counts: `tool_call`, `tool_result`,
  `assistant`, `thinking`, `user`, `tool_error`, `subagent_start`, `subagent_stop`, `session_start`,
  `session_end`, `aborted`, `permission_request`. Two footnotes: a `permission_request` count above
  zero is "a misconfiguration alarm, not a metric — a harness launched with its own gate on", and "A
  missing verb is recorded as missing, never synthesized."
- **Tools** panel — payload by tool, from the allowlist, with an anomaly note.
- Sessions table: `When · Harness · Project · repo · Agent · role · Model(s) · Runs · Events ·
  Errors · Tokens · Status · Id`. Status vocabulary: `running`, `stuck n/3`, `waiting: gate`,
  `idle Nm`, `handed off`, `closed`, `aborted`.

## Setup (project)

Header: project name, locator, a three-way segmented control **Settings / Persistence / Analytics**,
then **Archive** and **Rename**.

### Settings

- **Harnesses** — "which harnesses may run here, and which one an unattended agent gets by default".
  Table `Harness · Adapter · Provider · model · Agent default · (remove)`. A harness with no adapter
  is listed but not selectable. Exactly one row carries the **agent default**. An **Add harness**
  picker offers harnesses configured in Workshop but not yet added here ("Configured in Workshop,
  not yet added here."); empty state "Every harness with a working adapter is already here." Footer:
  "Enabled harnesses are offered to the router in the order listed; anything the router does not
  claim runs on the agent default. The routing logic itself is written once per harness, under
  Workshop → Harnesses."
  Empty state for the table: "No harnesses here yet — unattended agents in this project have nothing
  to run on."
- **Repos** — "a repo belongs to exactly one project — this is where that is decided". Each row
  carries origin, branch summary, activity, and a project chip with a caret (reassign). Footer:
  "Moving a repo re-tags every run, artifact and memory fact that came from it. The old tag stays on
  the record so history does not silently change project."
- **Extensions** — "pulled from the defaults in Workshop — switch one off and this project
  materializes without it". Seven tab groups: **agents, commands, hooks, linters, rules, skills,
  styles**. Per-group tri-state master toggle plus per-item toggles. Output styles carry a
  "set default" chip; the active default cannot be switched off without choosing another first.
  Footer: "Definitions are global; what is per-project is which of them this project gets. Switching
  one off here removes it from the materialized tree on the next run — it does not delete it."
- **CLI tools** — "installed in this project's container image — agents get exactly these, nothing
  else". Left: catalog search with a match count and per-row **Add** / **Remove**. Right: "In this
  project · n" with per-row removal. Footer: "Adding a tool rebuilds the image once, not per run. A
  tool an agent cannot find is the second most common reason a run stalls."
- **Base context** — "always loaded, every run in this project — exactly one, and there is no
  second". Token budget meter, `base.md` card with version, edit time, run count, **History** and
  **Save**. Footer: "Kept short on purpose: it is the one file every run pays for. Over budget
  usually means something belongs in a skill or a rule instead."

### Persistence (new)

"Everything this project has kept, in one place. Memory tiers decay on their own schedule; specs and
research stay until you delete them."

Three groups, each expanding into sections:

| Group | Sections | Notes |
| --- | --- | --- |
| Memory | Short-Term ("clears at session end unless promoted"), Long-Term ("promoted facts — survive the session"), Artifacts ("what runs left behind") | |
| Specs & Tasks | one section | Items carry the plan body and a nested task list; a task row navigates to the board |
| Research | one section | source and synthesis items |

Items expand to show their body. **Delete is offered on Long-Term and Artifacts only** — Short-Term,
Specs & Tasks, and Research have no delete control. Sections page at four items with a
"Show all n" / "Show fewer" toggle.

### Analytics (per project)

The same shape as global Analytics, scoped to the project: range tabs, four stat cards, trend chart,
breakdown table over the same five dimensions, Tasks (outcome bars gain **Landed after rework**),
tasks-by-role (`Role · Landed · Spend · Runs · Verified`), run times by workflow, memory retrievals
by tier, extension usage. Banner: "Every run, token and retrieval tagged #project — this project
only."

## Plan

Header: **All plans** toggle, a centred **Back / "Step n of 7 · Stage" / Next** stepper, an
**Outputs** toggle, then the plan title and origin.

**Stage strip, seven stages, each individually clickable:**

| # | Stage | Sub-label |
| --- | --- | --- |
| 1 | Inputs | what to add |
| 2 | Orient | repos indexed |
| 3 | Converse | n questions |
| 4 | Synthesis | spec draft, pass 2 |
| 5 | Recommend | spec + confidence |
| 6 | Decompose | what becomes a card |
| 7 | Approved | cards on the board |

**All plans rail** — In progress; **Drafts — rejected, kept here** (with confidence and open counts);
Approved · on the board. "New plan" starts with a goal, a target repo, and the repos involved — "the
goal is an input, not an output."

**1 · Inputs.** "What should this plan add?" One free-text goal, the target project, attached repos,
and **Start planning**. Footer: "One goal per plan. If this turns out to be two things, the
conversation will say so and you can split it there."

**2 · Orient.** Indexes the attached repos (symbols, call graph, history) before any question is
asked: "The conversation is only as good as what the index found, so this runs before any question
gets asked."

**3 · Converse.** Embeds the agent panel with header, research, plan tray, task link, and workflow
suppressed. This is the interviewer / researcher / auditor conversation.

**4 · Synthesis.** Two passes: requirements drafted, unsupported clauses cut, then read back for
gaps producing `open[n]`.

**5 · Recommend — the spec editor.** `spec.md` with version, requirement count, last edit, an
**unsaved** badge, and a confidence chip (`confidence 0.62 · open[2]`). Buttons **Revert**,
**History**, **Save & re-synthesise**.

- Requirements carry **stable ids** (`R-05`, `R-06`, …) in a fixed left column, one editable block
  each, RFC-2119 language. "Requirement ids are stable: a task already on the board keeps pointing
  at the requirement it came from, even after a rewrite."
- A requirement that closes a gap shows "closes open[1] — …" and a **Mark resolved** control.
- Per-section "+ Add requirement to §n".
- Outline rail with the five canonical sections: **Scope, Trust boundaries, Conflict resolution,
  Error conditions, Out of scope**; foot shows `open[n]` and "n requirement with no card".
- Footer: "Saving re-runs synthesis over the changed requirements only. Requirements already carried
  by a card on the board are marked, so you can see what a rewrite is about to contradict."

**6 · Decompose.** Three parts.

- **What becomes a card** — three choices: *The Spec* ("One card for the whole plan. The agent
  decomposes it at run time…", "coarsest · nothing to manage · 1 card"); *Every task* ("One card per
  task… Full visibility, and a board you now have to tend.", "finest · dependencies carried");
  *Spec + carve-outs* ("The spec rides as one card; the tasks you expect to be long get their own.",
  recommended).
- **Runs as** defaults bar — Workflow, Harness, Model, Effort. Model and Effort are disabled until a
  harness is chosen and default to `auto-route`; pinning either stops routing for that task. A
  warning appears while workflow or harness is unset: "Pick a workflow and a harness — every task
  inherits them."
- **Tasks from the spec** — columns `id · Task · Runs as · After · On the board as · (expand)`.
  Title is editable; "Runs as" summarizes overrides or reads "plan defaults"; "After" is the
  dependency; "On the board as" toggles between **its own card** and **rides on the spec card**.
  Expanding a row exposes per-task Workflow / Harness / Model / Effort and **Reset to defaults**.
  Footer states the resulting card count and **Approve — n to the board**.
  Copy: "Anything not carved out rides along on the spec card, and the agent decomposes it at run
  time. Carve out the ones you want to watch separately — usually the long ones and the ones you
  expect to get stuck." And, on effort: "On auto-route, the harness picks model and effort from the
  task complexity band. Pin either one and it stops routing for this task."

**7 · Approved.** "Approved — n cards on the board." Four stat cards (Questions asked/answered/
deferred, Requirements across five sections, final Confidence with carried `open[n]`, Cards).
A **What happened** log — one row per stage with description and duration. A **Cards created** table
(id, title, workflow · harness). Actions **Start a new plan** / **Open the board**. Copy: "The plan
is closed. The spec stays as the artefact the cards point at — editing it from here goes through the
board, not through this page." And: "A new plan starts empty — nothing here is carried forward
except the repo index, which is already warm."

**Outputs rail** (persistent): `spec.md` with counts and **Edit**; `tasks` preview with **Edit &
decompose**; a **recommendation** card with the confidence value, the open items, the ratchet
verdict ("one more elicitation pass before approval"), and **Approve — n to the board**.

## Interact (new surface)

Three panes.

**Sessions rail** (246px, collapses to a 40px dot strip that keeps live/selected state). Card
fields: live dot or state icon, name, harness, age, and a meta chip (`2 changed`, `clean`, `→ #1184`,
`discarded`). Per-row delete, no confirmation. Footer: **"A session is yours alone — no card, no
plan, no gate. Nothing here reaches the board unless you promote it."**

Empty state: "A session is a container, a branch and an agent you talk to directly. Start one to try
something without putting it on the board."

**Session states**: `open`, `promoted`, `discarded`.

**Center**: the agent panel, with cost shown and the research toggle wired. Opening research hides
the right rail — they share the same space.

**Changed this session** (320px): repo, base commit, branch (`interact/<id>`), per-file rows
(change marker, name, path, diff stat), file count. Empty state: "Nothing written yet — this session
has only read and run commands." A state-dependent note:

| State | Note |
| --- | --- |
| open | "This session has no card, so no approval gate and nothing in your Inbox. This panel is the only account of what it touched." |
| promoted | "This session was promoted to a card, so its diff now takes the normal gate. What you see here is the record of what it touched before that." |
| discarded | "This session was discarded. The container and branch are gone; the transcript stays for the record." |

**End the session**: **Commit to branch** — "Pushes `interact/<id>` to `<repo>`. You land it
yourself, later." (its caret opens the shared merge modal) — or **Discard** — "Kills the container
and deletes the branch. The transcript stays."

## Manage

One toolbar — segmented **Kanban / List / Graph / Timeline**, plus **Import task** and **Add task**.

### Kanban

Columns **Ready · In Progress · Testing · Reviewing · Pending Approval · Done**, each with a count;
a **Hide Done** toggle; "n cards · 3 in flight per person". Card decoration: live pulse with
workflow name and token count; a blocked marker naming the gate; a stuck ring (`stuck 3/3`); the
verify command on Testing; "Gate: reviewer agent" on Reviewing; evidence on Done.

Footer analytics — median dwell per column as a bar chart, and the reading: "The two slowest
columns are the two that need a human. Agents are not the constraint here — the median card spends
thirty-eight minutes being built and seventeen hours waiting to be looked at."

### List

Left: **Active** / **Inactive** with counts, then session cards (status dot, agent, role, tokens,
task, workflow and step, status chip, tool, run count). Footer: "Sorted by needs-attention, then
activity. Selecting one does not close the others — a session you stopped watching is not a session
you ended."

Right: the selected session. Live sessions show an iteration bar (`3/8`), tool errors against
baseline, token burn, last file write, and a verdict. Closed sessions show the outcome and "Nothing
is running — this is the record, not a live stream." Then a transcript pane. A stuck session raises
a guardrail banner — "Guardrail — kill & reassign after 3 stuck iterations", "Handoff drafted: 3
done, 2 remaining, 4 attempted, 1 open. The successor reads the payload, never this transcript." —
with **Hand off to `<agent>`** and **Let it run**.

Kind vocabulary: `run` (violet, pulsing), `wait` (ring), `idle` (dim), `bad` (red). Status strings
seen: `stuck 3/3`, `idle 3m`, `waiting: gate`, `running`, `verify`, `closed`, `handed off`,
`abandoned`.

### Graph (new)

A dependency DAG. Node = task card with title and status line ("Testing · unblocks 2", "Pending
Approval", "blocked: gate a-7741"). Edge colors carry meaning: ordinary dependency in grey, and
amber for edges into or out of a node **held by an approval you owe**. Caption: "Left to right is
dependency depth, not time."

Right rail **Unblocks most** — the same ranking the dispatch priority method uses, with the reading
"Two of the four cards holding up the most work are waiting on a human, not an agent — the same
story the dwell chart tells."

### Timeline (new)

Swimlanes grouped by workflow ("grouped by workflow · last 7 days"), a seven-day axis, and one bar
per card split into segments per board column, with the time-in-column value on the right. Legend
covers Ready, In Progress, Testing, Reviewing, Pending Approval, and "stuck or blocked". Caption:
"Bar length is wall-clock, not agent time. The widest bars are almost entirely amber and slate — a
card is on the board far longer than any agent is working on it."

## Review — QA (new surface)

"Tests, linters, LSP diagnostics and agent reviews for #project · last run 4m ago."

Schedule control **Manual / Push / Hourly / Daily**, plus **Refresh**.

Four finding groups, each with an icon, the tool it came from, and a pass/fail summary:

| Group | Tool |
| --- | --- |
| Unit tests | vitest · cargo nextest |
| Linters | clippy · eslint · ruff |
| LSP diagnostics | rust-analyzer · tsserver |
| Agent reviews | reviewer@2 · custom prompt |

Findings carry severity (fail / warn), a title, project and location, and a one-line explanation.
Each has a **Send to Inbox** / **Sent to Inbox** toggle.

Footer: "Not real-time — findings reflect the last scheduled or manual run. Sending a finding to
Inbox tracks it as a to-do; it stays listed here too."

## Dispatch

Three tabs: **Autorun / Schedules / Runs**, with a live/stopped pill and **Pause everything**.

### Autorun

Per-project switches plus an "All projects" master with a tri-state label (`All on` / `All off` /
`Mixed`) and an eligible count. Archived projects are locked off — "autorun cannot be turned on for
an archived project". A project can be **auto-suspended**: "Verify pass fell to 44%, under the 60%
floor — it comes back on its own when the number recovers."

Contract copy: "On means agents in that project pick up their own work and run it without you
starting anything. Off means every run begins with you, or with a schedule you wrote. There is no
third setting and no per-task exception."

**What holds it back when it is on** — a review-slot gauge ("3 of 4 review slots in use · 1 free")
with the reasoning: "A slot is one change you have not reviewed yet, not one agent. The median
developer reviews four changes a week; eight concurrent agents produce thirty-one. Autorun drains at
the rate you absorb, or it is just a way of generating a backlog faster." Then **Review debt**
(landed, unread; oldest), **Pauses at** (debt threshold), **Inbox budget** (n/hour), **Change
ceiling** (lines, files).

**Never autoruns** — five fixed exclusions, true even when the project is on:

1. Anything touching `migrations/**` — "A migration is append-only and irreversible in practice."
2. Any workflow containing a Gate node — "The gate is the point. Skipping it would be deleting it."
3. Anything over the change ceiling — "Past a reviewer's capacity, review degrades from semantic to
   syntactic."
4. A project under 60% verify pass — "Trust is measured, not assumed. It resumes on its own."
5. The first task of any plan — "You see what a plan produces once before it produces unattended."

**Stop all** — a confirm dialog naming exact scope (running agents killed at the next iteration
boundary; autorun switched off in n projects; n schedules skipped, not queued; branches, artifacts
and memory untouched), a **"Let each agent write its handoff first"** toggle (on: "Up to 30 seconds
each. A successor starts from the payload instead of re-deriving it."; off: "Immediate. Work in
flight is discarded and the next agent starts from the transcript."), and the footer "Reversible for
10 minutes — the handoffs are kept." Afterwards a banner reports what stopped and offers **Restore
previous state**.

### Schedules

Header meta: schedule count, fired, skipped; next firing and timezone; **New schedule**.

**Start work** builder:

- *What runs* — **Project** ("Runs every active agent in the project on whatever it is already set to
  work on… the agents' own assignments decide"; agents with nothing assigned are skipped) or
  **Custom** ("Assemble the run yourself. A spec sets the contract, a prompt narrows what to do with
  it — give it either, or both.") with agent, harness, project, an optional spec chip, and an
  optional prompt. Prompt note: "A prompt produces a run and an artifact, but no board task —
  nothing reaches the board without a plan."
- *Guardrails* (optional, per-schedule override): preset, max iterations, change ceiling, files
  touched, network tier, token budget, plus resolved permission pills. "Anything left unset falls
  through to Settings → Guardrails for #project. A ceiling reached here stops the run and splits it;
  it does not fail."
- *When* — **Run once, now** / **On a schedule** / **Hold**. "Schedules are yours. Autorun is the
  other path, and it is a per-project switch, not something you attach work to."
- A cron expression with a human readout and presets (Hourly, Nightly, Weekdays 09:00, Once at a
  time I pick), or attach the run to an existing schedule. Footnote: **"Overlap is skipped, never
  queued.** A job that runs longer than its own interval quietly stops running, so the skip count is
  a number on the card rather than a silence."

A misconfiguration banner fires when a schedule skips most of its firings, with **Widen the
interval**: "A schedule that skips every firing is misconfigured, which is why the skips are a
number and not a silence."

Schedule cards: live/paused state, cron and readout, workflow and step, last result (colored),
skipped count, and a duration sparkline.

Executions table — `Fired · Schedule · Result · Duration · Evidence`, results `passed` / `failed` /
`skipped`, with counters. "Recorded with their verify result — green or red, never merely
'finished'."

### Runs

Search ("Search every run — a path, a tool name, an event verb"), sort, date range, and KPIs:
spec-gap rate, noise reclassified, tokens per passing run. Table:
`When · Harness · Project · repo · Agent · role · Model resolved · Events · Errors · Tokens ·
Verify · Id`. Verify vocabulary: `running`, `passed`, `failed`, `failed ×n`, `waiting: gate`, `n/a`,
`aborted`. Subtitle: "Every run, scheduled or not · a schedule is just one way a run starts."

## Settings

Sections: **Guardrails** (populated), Inbox & notifications, Harnesses, Repositories, Store,
Appearance, Account. Sidebar footnote: "Settings are per install. Anything scoped to a project lives
in that project's base context instead."

Guardrails intro: "Defaults for every new run. A run can be given tighter limits than these; it can
never be given looser ones without an explicit override that is recorded on the run."

| Group | Controls |
| --- | --- |
| Stopping conditions | Max iterations; Token budget per run (the run is told at 80% "so it can spend what is left on a handoff"); Stuck detection (consecutive iterations with no file write); Kill & reassign on stuck ("The successor starts from the handoff payload, not the transcript.") |
| Parallelism | Max parallel agents (global); Max per project ("Counted against the same pool as the global cap, never in addition to it."); Priority method — `plan order`, `manual`, `unblocks-most`, `shortest first`; Tie-break `longest waiting`; Preempt a running agent ("the paused run keeps its handoff, not its context") |
| Change size | Lines changed ceiling; Files touched ceiling; On breach → `stop and split` |
| Permissions | Network tier for new agents ("Individual agent definitions may request less, never more."); Block unapproved system changes; Autopilot |

Footer: **Save defaults** / **Reset to shipped values** — "Changes apply to runs started after
saving. Nothing in flight is retuned underneath itself."

## Memory → Short-term (new screen)

"The context window. Nothing here is stored — it is rebuilt from scratch every iteration, which is
why what goes in it is a design decision and not a cache."

Left: live sessions with resident-token readouts, flagging any near the ceiling. Footer: "A fresh
context is a feature for the auditor and a cost for the builder. Both are deliberate."

Center: **Resident now** in fixed prefix order — "the order is the cache, so it never varies" —
base-context, rules in scope, skills loaded, the live plan, recalled facts, tool results, assistant
turns, each with a size and a tag (`cached`, `re-read`, `volatile`). The reading: "Four fifths of
the window is tool output. Everything authored… is under 4k, which is why the prefix stays cached
and why an unstable materialization is expensive out of proportion to its size."

**Compacted out** — "written to an artifact, replaced by one line naming it", each row `tool ·
description · size → artifact id`. "Compaction is the bridge between the two memories: short-term
drops it, and it becomes an artifact the agent can fetch again by name. Nothing is lost, only
moved."

Right: prefix cache percentage ("Stable while the materialized tree is stable. A reordered extension
invalidates the prefix for every run that follows it, not just the next one."); **What survives the
iteration** (facts written to long-term, artifacts, the plan and its checked steps — and not
"everything else, including the reasoning"); ceiling stats (compaction threshold, per-result
compaction trigger).

## Memory → Long-term

"Facts an agent recalls across runs. Promoted on evidence, ranked by provenance, and forgotten by
decay when nothing keeps asking for them." Scope is locked to the project — the second segment reads
**never cross-project**.

Confidence states: **verified**, **asserted**, **decaying**, **contradicted**. Each fact shows its
score and recall count; contradicted facts show no score.

Detail: the fact, its locator, author, promotion time, recall count, then **Why this is trusted** —
the passing verify that confirmed it ("Provenance beats recency, so this outranks any later
assertion"), the pages citing it, and the recall frequency. A confidence sparkline: "asserted 0.38 →
verified 0.94 · the jump is the verify, not the repetition", under the heading "decay is the
forgetting half; curation is the reconciling half".

**Curation semantics** (this closes the open question in `.specs/memory`): "Editing this makes it
yours, not the agent's. The page keeps both: the written fact stays as rev 1, your correction
becomes rev 2, and recall returns the curated one while provenance still points at the run."

Right rail: project totals; a **contradiction card** flagged at write with the two conflicting
values and their sources, offering **Adjudicate**; decay stats (fell below recall threshold,
promoted on a passing verify, median age of a decaying fact) with the note "Deciding what not to
remember is the half teams skip, after which retrieval quality degrades within a month"; and a
`locus memory explain` transcript.

## Memory → Wiki

Left: kind filter **All / Decisions / Concepts / Entities / Sources / Syntheses** with counts, an
**Ingest a document** action, and the page list with orphan flags. Footer: "Derived, then curated —
a path or a URL, never a blank page."

Page-kind definitions, verbatim:

| Kind | Definition |
| --- | --- |
| Decisions | "A fork, the option taken, and the cost of taking it. The only page kind that closes an argument." |
| Concepts | "An idea the codebase assumes. Named here so an agent can be told it once instead of inferring it every run." |
| Entities | "A thing the system has: a daemon, a table, a container. Orphans are flagged, because an entity nothing links to is usually a rename nobody finished." |
| Sources | "What was ingested, verbatim and unedited. Every assertion elsewhere points back to one of these." |
| Syntheses | "An answer assembled from several pages that exists nowhere on its own. The only kind an agent writes unprompted." |
| All pages | "…the only place the wiki can be checked as a whole: the link graph, the orphans, the assertions with no source." |

Center: page with revision, assertion and source counts, ingest and curation times; body; **Links
out** as wikilink chips; **Provenance** listing each source.

Right: a graph mini-panel ("Pages are nodes, wikilinks are edges — the canvas renderer,
repointed."); a contradiction card flagged **at ingest, not at query**, offering **Adjudicate** and
**Board card**; and a `locus wiki lint` panel (orphan pages, broken links, entities mentioned but
never given a page, assertions with no source, and the clean count). Footer: "The wiki is curated
prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else."

## Memory → Artifacts

Left rail in two sections: **Review artifacts** (with comment counts) and **Reference · never in the
inbox**. Kinds seen: `diff`, `walkthrough`, `image` ("OCR available"), `recording` ("12 keyframes
derived"), `diagram`, `finding`, `payload`.

Center: the viewer for that kind — "one viewer per kind · three entry points".

Right: **Comments steer the agent** — a thread mixing human comments and agent replies, a live
indicator when the run is still going ("run is still live · comment routed into the session"), and a
composer with **Send to session** / **Resolve**.

## Mail

Three panes. Left: tabs **All / Waiting / To you**, then threads carrying project, status, subject,
`from → to`, and a meta line (message count, or the blocking state and elapsed).

Status vocabulary: **waiting**, **open**, **replied**, **you**, **drained**.

Center: the thread, headed by a mail-wait banner — "`builder@4` is in `mail wait` — 8m of a 15m
timeout" / **"State is `waiting`, not idle. The idle guardrail will not fire."** Messages are tagged
with their verb: `mail send`, `mail read`, `mail reply`, `mail wait`. A live line reads "blocked
here · returns empty at 15m and the run resumes". Composer: reply as yourself, with **Drain** and
**Unblock**.

Right: participants with their run ids and states ("Different containers, one address space. Mail
survives a harness swap mid-project."); the verbs used (`send`, `read`, `reply`, `wait`, `drain`);
**What this becomes** — "Mail is a message between agents that both keep working. The moment
ownership transfers it stops being mail and becomes a handoff, with a payload the successor reads
instead of this thread." — with the drafted handoff artifact; and **Why you can read this** —
"Agent-to-agent mail is stored, not ephemeral. When a run goes wrong the question is usually what
one agent told another — and it was invisible until here."

## Workshop

Twelve entries. **Nine share one extension-editor template**; Providers, CLI, and Workflows are
bespoke.

### The shared extension editor

Types: **skills, rules, context, commands, hooks, styles, linters, agents, harnesses**.

- Left rail: icon, label, total, a blurb, **New `<singular>`**, a sort control, the item list, and a
  footer naming the storage unit ("One directory per skill, entry point SKILL.md.").
- Center: title and meta, **History** and **Save** (manual save — contrast Workflows, which
  autosaves), then up to six blocks: a frontmatter key/value table (field kinds text, select,
  number, toggle, chips — inferred from the field name), an autorouting section (harnesses only), an
  adapter-config table (harnesses only), a segmented field, a checklist, and the rendered file body.
- Right: **Materialization** — native and downgraded counts, a per-harness segment bar, the
  downgrade explanation, a **byte-deterministic** note ("Sorted order, no timestamps, no run id. The
  materialized tree *is* the prompt prefix, so an unstable one costs cache on every run that follows
  it."), and version history. Harnesses is the one type with no materialization rail.

Per-type notes worth carrying into specs:

| Type | Contract detail |
| --- | --- |
| skills | Lazy-loaded; a `budget_tokens` field "refuses to materialize above this". Downgrade inlines the description into base context and loses lazy loading. |
| rules | One glob per rule — "add a second rule rather than a second glob"; a priority field orders overlapping matches. The most downgraded type: concatenated into base context, firing on every file. |
| context | Exactly one per project; native everywhere; the fallback every downgrade lands in — "If it is over budget, something upstream was downgraded and should be fixed there instead." |
| commands | Argument-taking prompt templates — "a command with no args should be a skill". Downgrade materializes them as skills, losing argument validation. |
| hooks | One event each, a threshold and timeout, and an on-error choice ending in "exit 0, log, continue" — "a hook that fails a run has turned an optimisation into an outage." |
| styles | Exactly one active per harness; a roles checklist decides which roles get which style; "the largest single downgrade in the system" when merged into base context. |
| linters | Native 0 / downgraded 0 — "Not materialized into any harness — linters are a human-and-CLI surface by design, so there is nothing to downgrade." A violation choice of warn or fail — "a rule nobody can fail is a preference." |
| agents | Frontmatter plus a tool allowlist; the allowlist is the privilege set — "changing it rebuilds `locus/agent-<hash>` and invalidates the prefix cache. Editing the prose below rebuilds nothing." Native in every harness. |

The product's own count is **eight downgradable extension types plus Harnesses as the delivery
mechanism**.

### Workshop → Harnesses

Record: `identifier` ("must match the CLI on PATH"), `adapter` ("no adapter, no selection —
anywhere"), `providers` (chips, "only providers configured under Providers"), `default model` ("a
preferred model of that provider"), `default effort` ("what a run gets when nothing sizes it").

**Adapter config** — a free-form `Key · Value · Type` table with "Add config key": "free-form keys
the adapter reads — later config lands here without a schema change."

**Autorouting** — a per-harness switch. Off: "Autorouting is off for this harness. Every task runs
on the default model and effort in the record above… Turn it on to size tasks into bands." On: a
six-band table `Complexity · Model · Effort · Approval · When to use it`.

| Band | Approval | When to use it (abridged) |
| --- | --- | --- |
| xtra-low | — | Mechanical single-file edits. "No judgement required and none wanted." |
| low | — | One clear change against a known pattern. |
| medium | — | The ordinary task. "Most work lands here." |
| high | ✓ | Crosses module boundaries or has no obvious approach. |
| xtra-high | ✓ | Architecture, protocol semantics; "a wrong answer is expensive to unwind". |
| max | ✓ | "Reserved for the plan itself and for tasks that stuck twice already." |

Semantics: "A band with no model set never receives work: the task falls to the next band up. Sizing
happens once, when the card reaches the board — not per iteration." And: "Models come from this
harness's providers, so an alias set under Providers is what you pick here. A ticked band waits for
you before it starts."

The downgrade vocabulary is gone from this screen: "All eight extension types are supported on every
harness. What differs is the mechanism, and the mechanism is the adapter's problem — not yours."

*Discrepancy to resolve:* band efforts include `minimal`, while Plan → Decompose cycles
`low / medium / high / xhigh`.

### Workshop → Providers

Left: providers with a status dot — **ok / warn / off** — and a preferred-model count. Footer:
"Secrets live in the OS keychain. Locus stores the reference and the model list, never the key."

Center: **Authentication** ("one credential per provider — every harness pointed at it uses this
one") — method segmented **OAuth / API key / None**, a masked secret with **Reveal** and
**Replace** and a `keychain` tag, a `base_url` override ("override for a proxy or a gateway"), and a
verify line ("verified 11m ago · 327 models listed", or a warning such as "token expires in 6 days —
re-consent to avoid a failed dispatch").

**Preferred models** — `Model · Alias · Context · In / out per M · In selector · (remove)`, with a
catalogue search showing "n of N match". "An alias is what the model selector shows from here on —
for every harness pointed at this provider. Without one, the selector shows the full id." And: "n
models offered · n preferred · the rest never appear in a selector."

Right: a preview of the selector as the user will see it ("Nothing preferred yet, so this provider
offers no models to any harness."); harnesses using the provider — "Removing this provider unsets
the model on each of them rather than failing their next run."; and 30-day spend.

### Workshop → CLI

"A tool switched on here can be added to any project. Off means it is not in the image at all — an
agent cannot reach for something you have not given it, and a missing tool is the second most common
reason a run stalls."

Built-in tools grouped by category — **Source control, Search & files, Rust, Database, Web &
network** — each with per-tool toggles and a tri-state group master. **Your own**: uploaded tools
with signature state, an overflow menu, and a dropzone ("Drop a binary, a script, or a `tool.toml`"
/ "Or paste an install line — cargo install, npm -g, pipx, a release URL. It is built into the image
once and pinned by digest.") with `install` and `verify it landed` fields.

Signing gate: **"An unsigned tool is available to read-only roles only. Sign it, or accept that impl
runs will not see it."**

Right: **Image** — "Enabled tools are baked into the base image, not installed per run. A change
rebuilds it once." — with size and last rebuild; and **Most reached for** with the note "A tool
nobody reaches for is a tool to switch off."

### Workshop → Workflows

Header: editable title, **Visual / Governance** switch, an autosave chip ("saved 2s ago"), and an
**Inspector** toggle. No Save, no Validate.

**All workflows** list grouped **Published / Draft / Archived**, each row carrying node count, edit
time, and references ("referenced by 1 schedule"). Footer: "Presets are workflows too. Copy one
rather than starting from an empty canvas."

**Visual** — palette **Agent, Task, Loop, Condition, Gate, Verify** (Verify tagged *req*), plus
presets that "expand into ordinary nodes, so it can be edited rather than configured". **There is no
Goal node.** The condition inspector offers an expression builder, the compiled expression, and a
validity chip: "total · evaluable in the core · reproducible from stored events". Operands are
enumerated — `verify.passed`, `verify.exit_code`, `iteration`, `elapsed`, `tokens.used`,
`events.count(tool_error)`, `events.last(kind)`, `artifact.exists(kind)`, `task.status`,
`mail.pending` — under the rule **"No code, no model, no I/O — anything this cannot express is a
Gate."** The inspector ends by pointing at Governance: "Guardrails and success criteria live in
Governance — they belong to the workflow, not to one node." Canvas caption: "No model in the
orchestration path — the graph decides."

**Governance** — three sections:

- **Goal** — "the guiding statement — every node is judged against it, and it is also the
  termination condition", with "Link to a plan instead". Example note: "the loop exits when the goal
  is met, not when the agent says it is finished."
- **Guardrails** — titled markdown prompt cards, "read by the run while it is in flight". The
  composer note: "Goes into the run's context before the first iteration, and again after any
  reset."
- **Success criteria** — "checked after the run — all must hold before the workflow reports done".
  Columns `(check) · Kind · Criterion · Checked by`; kinds **command / assertion / human**; checked
  by **core** or **gate — you**. Closing rule: **"A criterion the core cannot check itself becomes a
  gate: it goes to your inbox with the evidence attached rather than being marked passed on the
  agent's word."** Results land on the run, not here.

## The agent panel (`AgentPanel.dc.html`)

Matches `.specs/agent-interface` closely. Header: session locator, editable name, optional linked
task and workflow, token/cost readout, a Research/Changes toggle with unread count, and an overflow
menu carrying **New session** (`/new`), **Compact context** (`/compact`), **Clear context**
(`/clear`) plus disclosure controls — Thinking (Summary/Full/Hidden), Tool calls
(Expanded/Collapsed/Hidden), User cards, Token/cost, Research pane.

Stream: a derived live pill (idle / working / waiting on you); collapsible user cards; agent turns
with thinking blocks ("Thought for 8s · 3 considerations"), inline file links, code blocks, tool-call
cards with status (`completed`, `tool_error`, retries), `plan_update` dividers, edit cards with diff
stats and approval state, **checkpoint rows with Restore** (and an undo banner: "Workspace reverted
to Checkpoint 7 — 2 files, 13 lines. The transcript is kept."), elicitation summaries, and consent
rows.

Blockers dock at the bottom over a scrim and can minimize to a pill: a **permission gate** rendering
the unified diff with **Approve** / **Decline** and "Approve the remaining n edits in this turn"; and
an **elicitation** card with choices, an optional free-text field, and **Send answer** / **Decline**.

Footer: a collapsible **plan tray** (progress bar, step counter, current step, per-step status), the
composer with slash-command popover, a **run-config chip** (harness · model · effort) opening a
Harness / Model / Effort modal ("Applies to the next turn"), a **Manual / Auto** gated-mode toggle,
and a **context-window chip** opening a breakdown (base context, memory catalogue, returned tool
docs, transcript, active plan) with per-row actions.

Research pane (right, 340px): seeded findings ("2 of 4 came from the plan"), per-finding kind badges
(**seed** / **this run**), source citations, claim → source links, and a footer "refac nominates 3
for long-term memory".

Permission modes: **Gated** ("Every irreversible action stops here for your approval — file edits
arrive as a diff you accept or refuse") and **Bypass** ("No prompts. The container-enforced allowlist
is the only limit, and every edit still leaves a checkpoint").

*Discrepancy:* the mockup ships `showCost` on; `.specs/agent-interface` defaults token/cost off.

---

## Gap table

| Surface | Mockup | Specs | Code |
| --- | --- | --- | --- |
| Shell / rail | Two title-bar pills, Project + Cross-Project groups, five renamed categories | `desktop-application-shell` describes the superseded rail | `apps/desktop/src/shell/Shell.tsx:19-51` collapses 31 routes onto a 14-view union |
| Plan pipeline | 7 stages, stepper, Approved summary | `planning-module` says 8; `design-desktop` says 9 | `services/planning.rs:9-32` ships 9 |
| Spec editor | Stable `R-xx`, `open[n]`, confidence, partial re-synthesis | Not written down anywhere | `EditableSpec` + `changed_ids` already implement it |
| Decompose | Granularity, per-task workflow/harness/model/effort, auto-route, card count | `design-desktop` covers granularity only | Decomposition present; per-task routing overrides absent |
| Interact | Board-less sessions, promote/commit/discard | No spec | `agents.sessions.board_task_id` exists; lifecycle actions absent |
| QA review | Four groups, schedule control, Send to Inbox | No spec | Nothing |
| Analytics | Ranges, five dims, funnels, p90, retrievals, extension usage | `dashboard-metrics` describes a different screen | Raw data exists; no projections |
| Setup → Persistence | Grouped tree, tier-scoped delete | No spec | Nothing |
| Manage → Graph / Timeline | DAG with approval-held edges; per-workflow swimlanes | No spec | `board.task_transitions` carries the data |
| Autorun policy | Verify floor, review slots, five exclusions | `design-desktop` names autorun only | `runtime/dispatch.rs` has `AutorunState`, not the policy |
| Schedules | Per-schedule guardrail overrides, project vs custom mode | `schedules` covers cron + skip-not-queue | Overrides absent |
| Stop all | Scope dialog, handoff toggle, 10-minute restore | `design-desktop` | `StopAllSnapshot` matches |
| Settings guardrails | Four groups incl. parallelism and preemption | `guardrails` | `DispatchPolicy` matches |
| Harness record | Adapter gate, providers, defaults, adapter config, six bands | `design-desktop` has bands; no adapter config | `runtime/routing.rs` has bands; adapter config has no home |
| Providers | Auth, aliases, selector preview, spend | `design-desktop`, `desktop-workshop-runtime` | `store/providers.rs` + migrations `0012`/`0014`; no spend rollup |
| CLI tools | Groups, signing gate, image bake | `marketplace-*`, `design-desktop` | `services/tools.rs` matches |
| Workflows | Visual/Governance, no Goal node, operand set | `workflow-canvas` still has a Goal node | `services/workflow.rs` matches Governance |
| Short-term memory | Prefix anatomy, compaction bridge | No screen spec | `tool-compaction` covers the mechanism |
| Long-term memory | Four confidence states, adjudication, dual-revision curation | `memory` leaves curation open | Decay/promotion present; adjudication absent |
| Wiki | Five kinds + All, lint panel | `wiki` lists six kinds incl. `overview` | — |
| Artifacts | Review vs Reference, comments to live session | `artifacts` matches | — |
| Mail | Full surface, wait/drain/unblock | `mail` covers mechanics, not the screen | — |
| Agent panel | Full contract | `agent-interface` matches; reference path is dead | Panes render events |

## Consequences

Six surfaces have no spec at all (Interact, QA, Analytics as drawn, Persistence, Graph, Timeline,
Mail-as-screen). Two shipped contracts contradict the mockup (nine-stage planning, the Goal node).
One large policy surface — autorun — is drawn in detail and specced in a sentence. Everything else
is contract detail the specs can absorb without changing direction.
