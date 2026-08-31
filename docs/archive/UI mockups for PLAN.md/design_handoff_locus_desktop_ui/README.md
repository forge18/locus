# Handoff: Locus desktop UI (Nocturne shell)

## Overview
A single high-fidelity HTML mockup of the **Locus** desktop app — the multi-project, multi-agent
orchestration client described in `PLAN.md`. It covers the whole application shell (title bar,
category rail, per-category tab bar, running-agent strip) and **thirteen** interior screens across
seven categories: Inbox/Status, Plan, Develop, Automate (Kanban + Agents), Review (Telemetry, Runs,
Artifacts) and Workshop (Extensions, Agent definitions, Workflow, Harnesses), plus the Wiki.

Target repo: `/Users/forge18/Repos/locus` — implement inside `apps/desktop`
(**Tauri 2 + SolidJS + Vite + TypeScript**, per `apps/desktop/package.json`). `src/App.tsx` is still
the Tauri scaffold, so this is a greenfield UI build in an existing, empty-but-conventioned frontend.
Honor the repo's own notes: `src/ui/README.md` (shadcn-solid components copied in, over Kobalte
primitives — owned locally, not depended on), `src/panes/README.md` (pane manager: Agent Panes =
typed event streams, Shell Panes = real PTY via xterm.js, editor pane, minimize-to-tile; one webview
per window), and `src/workflow-canvas/README.md`.

## About the Design Files
`Locus.dc.html` is a **design reference created in HTML**, not production code. It is a prototype
of intended look and behavior. Do **not** port its markup. Recreate these screens as SolidJS
components in `apps/desktop/src`, using the repo's established patterns: shadcn-solid/Kobalte
components in `src/ui`, product surfaces in `src/panes` and `src/workflow-canvas`, Tauri
`invoke`/event subscriptions for data. The mockup's own component runtime (`support.js`,
`<x-dc>`, `<sc-if>`, `<sc-for>`) is scaffolding for the design tool and has no analogue in the app.

## Fidelity
**High-fidelity.** Final colors, type sizes, spacing, densities and copy. Recreate pixel-closely,
but express the values as Solid/CSS variables rather than inline styles (the mockup uses inline
styles only because of its authoring environment). The design is deliberately **dense** — it is a
professional tool at 1440×900; do not "breathe" it out.

## Design Tokens
Declared once on `:root` in the mockup; carry them over verbatim as CSS custom properties.

| Token | Value | Role |
| --- | --- | --- |
| `--bg` | `#1d2731` | app ground |
| `--bg-deep` | `#151d25` | title bar, rail, footers, strip |
| `--sf` | `#22303c` | surface / card |
| `--sf2` | `#293947` | raised / selected surface |
| `--sf3` | `#314454` | chips, your-message bubble, active tab |
| `--blue` / `--blue-lit` | `#083c5d` / `#0d5480` | agent avatar grounds, graph nodes |
| `--ac` | `#ffbb39` | accent (amber) — selection, focus, "needs you" |
| `--tx` | `#eef2f6` | primary text |
| `--mu` | `rgba(238,242,246,.56)` | secondary text |
| `--mu2` | `rgba(238,242,246,.34)` | tertiary text / metadata |
| `--line` / `--line2` | `rgba(238,242,246,.10)` / `.18` | hairlines, borders |
| `--ok` | `#4fa07f` | pass / added lines |
| `--bad` | `#d4614f` | fail / removed lines / stuck |
| `--fm` | `'JetBrains Mono', ui-monospace, Menlo, monospace` | all identifiers, paths, ids, numerics |

Body font: **Inter** (system-ui fallback), weight 400 with **500** for every emphasis — never 600+.
Mono is used for locators (`locus://…`), branches, file paths, model names, session ids, counts in
tables, and code.

Type scale actually used: 9–10px uppercase section labels (letter-spacing .11–.12em, color `--mu`),
10.5–11px metadata, 11.5–12.5px body/rows, 13–15px screen titles, 17–27px metric numerals.
Radii: 5–6px (chips, tabs, small controls), 7–9px (cards, panels), 11px (window). Gaps: 2/4/6/9/14px.
Shadows: only `0 8px 22px rgba(0,0,0,.45)` / `0 10px 26px rgba(0,0,0,.5)` on canvas nodes, and
`0 0 0 1px rgba(238,242,246,.13), 0 30px 80px rgba(0,0,0,.6)` on the window. Selection is expressed
as `box-shadow: inset 0 0 0 1px var(--ac)` over `--sf2` — an inset ring, never an outer glow.

Icons: **Phosphor** (regular + fill), 9–19px. Two keyframes only:
`pulse` (2s, opacity 1→.25, live dots) and `blink` (1.1s, hard on/off, text carets).

## Shell (present on every screen)
Window: 1440×900, radius 11px, `--bg`, column flex.

1. **Title bar** — 38px, `--bg-deep`, bottom hairline.
   - macOS traffic lights (11px circles: `#ed6a5e`, `#f4bf50`, `#61c554`), then `LOCUS` at
     12px/500, uppercase, letter-spacing .14em, `--mu`.
   - Centered **locator bar**: 520×24, `--sf`, radius 6, hairline; magnifier icon, mono
     `locus://` in `--mu2` then the current path in `--mu`; right-aligned `⌘K` in a 4px-radius
     hairline box. This is the app's addressing scheme — every object has a `locus://` URI.
   - Project filter: `All projects · 4` with funnel icon in accent.
   - Right: pulsing accent dot + `8 running`.
2. **Category rail** — 78px, `--bg-deep`, right hairline, 6px padding, 2px gaps.
   Items (Phosphor 19px over a 9.5px label): **Inbox** (`tray`, badge `3` — 15px accent pill,
   text `#1d2731`, 700/9.5px, absolutely positioned top 5 / right 9), **Plan** (`compass`),
   **Develop** (`code`), **Automate** (`lightning`), **Review** (`chart-bar`),
   **Workshop** (`wrench`), **Wiki** (`book-bookmark`). Active item: `#293947` +
   `inset 0 0 0 1px rgba(255,187,57,.55)` + accent text; inactive `rgba(238,242,246,.56)`.
   Rail foot: `git-branch` and `user-circle` glyphs in `--mu2`.
   The rail is **category-level**: it highlights by the *category* of the current view, so drilling
   into a sub-screen keeps its category lit.
3. **Tab bar** — 36px, gradient `linear-gradient(var(--sf), var(--bg))`, bottom hairline.
   Left: current category label (12px/500, uppercase, .1em, `--mu`). Then only the tabs belonging to
   that category (3px 10px, radius 6; active = `#314454` + `inset 0 0 0 1px rgba(255,187,57,.5)` +
   `--tx`). Right: the mono locator for the current view + `arrows-out-simple`.
4. **Strip** — 46px footer, `--bg-deep`, top hairline. A vertical `STRIP` label then one compact
   card per running agent (project · agent · role over status · tool · tokens). Cards: `--sf`,
   radius 7, hairline — red-bordered when stuck, dimmed with a `terminal-window` icon for your own
   shell ("no agent · no cost"). Right: `sorted by needs-attention, then activity`.
   The strip is minimize-to-tile for panes; ordering is needs-attention first.

### Navigation model (implement exactly)
- Views → category, label, locator:
  `inbox`/`status` → dashboard "Inbox"; `plan` → Plan; `wiki` → Wiki; `develop` → Develop;
  `board`/`sessions` → Automate; `telemetry`/`runs`/`artifact` → Review;
  `extensions`/`agents`/`canvas`/`harnesses` → Workshop.
- Tabs per category: dashboard `[Inbox, Status]`; Automate `[Kanban, Agents]` (in that order);
  Review `[Telemetry, Runs, Artifacts]`; Workshop `[Extensions, Workflow, Harnesses]`;
  Plan / Wiki / Develop have none.
- Rail click lands on each category's first view: Inbox → `inbox`, Automate → `board`,
  Review → `telemetry`, Workshop → `extensions`.
- **`agents` (agent definitions) is a drill-down of Extensions, not a tab.** It is entered from the
  `agents` card on the Extensions screen; while it is open the **Extensions** tab stays lit and its
  sidebar shows a `← Extensions` back link. Do not add an Agents tab to the Workshop bar (the
  Automate "Agents" tab is a different thing — the live session list).
- Only two pieces of view state in the mockup: `view` and the selected session index.

## Screens

### 1. Inbox (dashboard · default)
Two panes. **Left 392px**, right hairline.
- `NEEDS YOU` label in accent + `3 items · silence is the default`. Three cards (radius 7):
  the first is selected (`--sf2` + accent inset ring) — `seal-check` fill icon, "Gate — approve
  plan before implementation", age right-aligned in `--mu2`, then a subline with project · agent ·
  mono branch. Others are `--sf` + hairline: a `question` icon "locus ask — which migration path?"
  and a `warning-octagon` in `--bad` "Guardrail — kill & reassign, 3 stuck iterations".
- `RESOLVED TODAY` label, then three rows at `opacity:.6` (icon, one-line title, age).
- **Right pane**: header with `plan` accent tag + 15px title, then a metadata row (mono locator ·
  agent · role · Gate: human). Body: uppercase accent `PLAN` label and four numbered steps at
  12.5px/1.6 with mono inline paths; an `info` callout in `--sf`; then "Comment steers the agent
  that made it" over a 64px `textarea` (`.input`, `--sf`). Footer bar (`--bg-deep`, top hairline):
  primary "Approve & release the loop", secondary "Send back with comment", right note
  "Resolves here · the work opens where the work lives".
- Behavior: the inbox is the only interruption surface; approving resolves in place and releases the
  agent loop; the item's *work* opens where the work lives (Plan/Develop/Review).

### 2. Status (dashboard)
Scrolling column, 15/18px padding, 14px gaps.
- **Six metric cards** in a 6-col grid: Running 8 (`4 panes · 4 strip`), **Waiting on me 3**
  (accent card: `--sf2` + accent ring, accent label and numeral, `oldest 26m`), Verify pass 71%,
  Cache read 84%, Tokens today 4.2M, Guardrail trips 5 (`1 kill & reassign` in `--bad`).
  Numerals 27px/500; unit suffixes 15px in `--mu`; label 10px uppercase .1em.
- **Runs by hour** (1.55fr): 12 stacked bars, 118px tall, 5px gaps — accent = passed, `--bad` =
  failed, `--blue-lit` = aborted, stacked bottom-up; mono hour axis 08/11/14/17/20.
- **Wants attention** (1fr): three rows on `--bg` — stuck (red inset ring, `warning-octagon` fill,
  `Reassign` link in accent), idle (`moon`), waiting (`hourglass-medium`, "waiting: gate — not idle").
- **Project table** (`.table`): Project / Repos / Running / In review / Verify / Tokens today /
  Cache / Last event for tapestry, loom-db, weaver, texere. Verify colored `--ok`/`--bad`; numerics mono.

### 3. Plan
Three panes: 216px list · flexible conversation · 296px outputs.
- **List**: block primary "New plan" + note that a plan starts from a goal, a target repo and the
  repos involved. Sections `IN PROGRESS` (selected card with accent ring, `circle-notch` +
  "step 5 · audit", project right), `DRAFTS — REJECTED, KEPT HERE` (with confidence/open counts),
  `APPROVED · ON THE BOARD` (dimmed, `--ok` "6 tasks landed"). Footer: "Nothing reaches the board
  until one approval at the end."
- **Conversation**: an 8-step breadcrumb (Inputs → Orient → Converse → Synthesise → **Audit** →
  Recommend → Override → Approve): done steps get an `--ok` check, the current step is an accent
  pill, future steps are `--mu2`. Messages: 22px rounded-5 mono initials avatar (`--blue` for
  agents, `#5c4413` for the auditor) + role caption + bubble (`--sf`, max 600px); your replies are
  right-aligned on `--sf3`, max 560. Inline **scope decision** card (accent ring, `arrows-split`):
  "resolves inline, not as a separate gate" with "Widen scope" / "Keep out, note as open". Auditor
  finding uses a red-tinted border. Live line: pulsing dot + "interviewer is re-opening question 14
  of 14". Footer: fake input with a blinking accent caret + mono `ACP · session/prompt`.
- **Right rail**: `DRAFT OUTPUTS` — spec.md, tasks (4 numbered), tool list (mono `.tag` chips,
  `+ pgvector` as `.tag-outline`), and a **recommendation** card (accent ring): 21px `0.62`
  confidence, `open[2]`, ratchet note, block primary "Approve — 4 tasks to the board".

### 4. Develop
Three columns: 206px file tree · editor · **252px git panel**.
- **Tree**: branch header (`git-branch` accent + mono `agent/8f21-notify` + caret), indented rows
  (11.5px; 20px/34px indents), selected file on `--sf2` radius 5 with an accent `M` badge;
  footer "Linked repo · your own checkout at ~/Repos/tapestry".
- **Editor**: 30px tab bar on `--bg-deep`; active tab is `--bg` with `inset 0 2px 0 var(--ac)`
  and a close `x`; right side `collapseUnchanged` + accent `2 chunks`. Body: **side-by-side diff**,
  mono 11.5px/1.65, 34px right-aligned gutter in `--mu2`, ± column 12px wide; removed rows
  `rgba(212,97,79,.14)`, added `rgba(79,160,127,.16)`; collapsed regions are `⋯ N unchanged lines`
  strips on `rgba(238,242,246,.03)`. Left header `HEAD · main` (`--mu2`), right
  `agent/8f21-notify · builder@4` (accent). Keywords `#8fb8d6`, comments `--mu`.
  Footer 52px: secondary "Revert chunk", primary "Open PR from this branch", mono
  `rust-analyzer · 0 errors · 2 hints`, right note "Reviewing what an agent changed is the primary
  editor surface".
- **Git panel** (`--bg-deep`, left hairline):
  header `GIT` + mono `2↑` (accent) `0↓` (`--mu2`) + `arrows-clockwise`;
  branch block (`git-branch`, mono branch, "from main · pushed by builder@4 6m ago");
  scrolling body with `STAGED 2` (+ "Unstage all" accent link) and `UNSTAGED 2` (+ "Stage all"):
  rows are a status letter (`M` accent, `A` `--ok`, `?` `--mu2`, 9px wide), mono truncated path,
  right-aligned `+N` / `−N` counts; the current file's row sits on `--sf2`;
  then `HISTORY · this branch` — three commits as 7px dots (newest accent with a
  `0 0 0 3px rgba(145,132,217,.16)` halo, older `--mu2`) + subject + mono `sha · author · age`;
  footer: commit-message field, primary **Commit** + secondary **Push**, and the note
  "Working tree is your own checkout — the agent pushed to the branch, you decide what lands."
  Behavior to build: stage/unstage per file and per hunk, commit, push, and open-PR; the panel is
  read-write on *your* checkout while the branch is the agent's.

### 5. Automate · Kanban (first tab)
Header: "Fixed columns across every project", then `prohibit-inset` + "blocked is a status, not a
column", right-aligned project `.tag-neutral` chips. Body: **6 fixed columns**
(Ready 3 · Building 2 · Testing 1 · Reviewing 2 · Waiting For Approval 1 · Done 1) in a 6-col grid,
9px gaps. Column head: 10px uppercase label (accent for Waiting For Approval) + count in `--mu2`.
Cards: `--sf` + hairline, radius 7, 11.8px title, then a 10px meta line (accent project · repo,
mono verify command, `reviewer@2 · read-only tools`, `Gate: reviewer agent`). A blocked card shows
a red `prohibit-inset`; a stuck card gets a red inset ring and `stuck 3/3 · 102.3k`; the
approval-waiting card gets the accent ring and "an inbox item, not a place to go looking"; Done is
`opacity:.86` with `--ok` "evidence: 2 runs, 41 events". Columns are fixed — no add-column affordance.

### 6. Automate · Agents (second tab)
Live session list + transcript. **Left 356px**: header `AGENTS · 8 running · one session each` with
funnel + accent sort icons. Cards (`#22303c`, radius 8): status dot, project 12px/500, mono agent,
role, right-aligned mono tokens; task line 11.5px at 76% opacity; then a status chip, mono current
tool, and right-aligned run count. Selected card: `#293947` + `inset 0 0 0 1px rgba(255,187,57,.55)`;
stuck cards carry a red hairline. Footer: "Sorted by needs-attention, then activity. Selecting one
does not close the others — a session you stopped watching is not a session you ended."
**Right**: header (dot, project, mono agent, role, truncated task, status chip, mono locator,
`arrows-out-simple` = detach to its own Tauri window, `minus` = minimize to strip). Body is a mono
11.5px/1.68 event stream, colored by verb — accent for tool calls, `#8fb8d6` for thinking, `--ok`
pass, `--bad` error — ending in a prompt line with a 7×14px blinking accent block cursor.
Conditional footers: **stuck** → red-tinted guardrail card ("kill & reassign after 3 stuck
iterations", handoff summary, "Hand off to reviewer@2" / "Let it run"); **waiting** → `--sf` card
with `hourglass-medium`, "Waiting ≠ idle." Status bar: mono "PTY attached from the host · one
session per terminal" + run id.
Data per session: project, agent, role, task, status/kind (running · idle · waiting · stuck),
tokens, tool, runs, prompt, run id, and the transcript lines.

### 7. Review · Telemetry
Scrolling. Search bar with mono-ish query `tool_error`, blinking caret and the note "every event,
every session · BM25 over the normalized log"; `.tag-outline` filter chips (`verify: failed`, `30d`)
+ "Reset filters". Four metric cards (Sessions 641, Events 154,385, **Tool errors 2,190** in `--bad`
with a red hairline, Output tokens 77.46M) plus a 1.5fr sparkline card of 16 accent bars at 85%
opacity. Then a 3-column 434px band:
- **Filters** — grouped facet chips on `--sf3` with counts in `--mu2`: harness, capture source,
  project, agent · role, model tier, verify (active chip = accent tint + accent inset ring), arbiter
  class, branch. The branch group states the invariant: `main 0` at `opacity:.5`.
- **Actions** — the canonical event vocabulary as mono rows: 132px name, 7px track
  (`rgba(238,242,246,.06)`) with an accent (or `--bad`) fill, right-aligned count. Includes the
  `2 permission_requests` alarm callout and the "missing verb is recorded as missing" note.
- **Tools** — same row pattern at 112px labels, with an anomaly note.
Bottom: `SESSIONS (300)` `.table` — When / Harness / Project · repo / Agent · role / Model(s) /
Runs / Events / Errors / Tokens / Status / Id; numerics mono and right-aligned; status colored
(accent running, `--bad` stuck/aborted/handed off, `--ok` closed, `--mu` waiting).

### 8. Review · Runs
Search ("a path, a tool name, an event verb"), a `.seg` segmented control (Today / 7d / **30d**
active in accent), counts `612 runs · 300 sessions · 4 projects`, and three right-aligned stats
(spec-gap rate 9%, noise reclassified 6, tokens/passing run 136k). Then `RUNS (612)` `.table`:
When / Harness / Project · repo / Agent · role / Model resolved / Events / Errors / Tokens / Verify / Id.

### 9. Review · Artifacts
Three panes. **222px list**: `REVIEW ARTIFACTS` — one viewer per kind (diff, walkthrough, image with
OCR, recording with derived keyframes, diagram), then a dimmed `REFERENCE · NEVER IN THE INBOX`
group (finding, payload). **Center**: header (`diff` accent tag, mono file name, locator, "one
viewer per kind · three entry points") over a unified diff — `@@` hunk headers in `--mu2`, 26px
gutter, added/removed tints as in Develop, and the commented line marked with
`inset 3px 0 0 var(--ac)`. **306px right rail**: `COMMENTS STEER THE AGENT` — your comment
(`--sf`, 16px mono-initial avatar) and the agent's reply (`--sf2` + `--line2` ring) with a pulsing
"run is still live · comment routed into the session"; footer textarea + "Send to session" /
"Resolve".

### 10. Workshop · Extensions
Header: "The one surface" + "Eight extension types, authored once here, materialized fresh into
every runtime at run start", search field, primary "New". **4×2 grid of type cards** (radius 9):
icon + lowercase type name + a 20px count, a 11px description, and a footer line of native vs
downgraded counts (`--bad` when the downgrades dominate). Types: agents 6, skills 12, rules 14,
base-context 1, commands 9, hooks 7, output-styles 3, linters 5. **The `agents` card is the
selected/entry card** (accent ring, pointer cursor, accent `arrow-right`) and navigates to the agent
definitions screen. Below: `RECENTLY EDITED` list (type `.tag-neutral` chip min-width 82px, file,
change summary, age) and a `MATERIALIZATION` card with an amber hairline: byte-determinism note plus
three figures (88 entries / **27** downgrades / 84% cache read).

### 11. Workshop · Agent definitions (drill-down of Extensions)
**196px sidebar**: `← Extensions` back link (accent, 10.5px, `arrow-left`), then `AGENT
DEFINITIONS` — builder (selected, `--sf2` + `--line2` ring), reviewer, interviewer, researcher,
auditor, keeper, each with a mono version; footer "Markdown plus a tool list. No canvas, no compile."
**Main**: header `builder.md` + mono "v4 · edited 2h ago · used by 5 sessions", "Diff v3" /
"Save as v5". Body: a frontmatter block (`--sf`, `border-left:2px solid var(--ac)`, mono 12px/1.72,
keys in `#8fb8d6`: harness, model_tier, tools, skills, rules, memory_scope) followed by Inter prose
at 13px/1.65, max 660px. Footer note: materialized to `/locus/config/agents/` for 11 harnesses,
3 downgraded.

### 12. Workshop · Workflow
Three panes: 180px palette · 890px canvas · flexible inspector.
- **Palette** (`--bg-deep`): draggable node chips (`cursor:grab`, `dots-six-vertical` handle) —
  Goal (amber hairline), Agent, Task, Loop, Condition (`#8fb8d6`), Gate, Verify (marked `req`);
  then `PRESETS` (Ralph loop, Review pass) on `--sf2` and the note "A preset expands into ordinary
  nodes, so it can be edited rather than configured."
- **Canvas**: 24px dot grid (`radial-gradient(rgba(238,242,246,.085) 1px, transparent 1px)`), an
  SVG edge layer with four arrow markers (neutral, accent, `--ok`, `--bad`), a dashed loop-back
  edge, a dashed rounded rect grouping the loop, and absolutely positioned node cards (radius 9,
  tinted 5×9px header strip with icon + 9.5px uppercase kind + right-side state). Nodes: Goal
  (approval gate, "also the termination condition"), Task, **Agent** (selected: 2px accent ring,
  pulsing `iter 3/8`, chips for role/tools/net/path scope), Verify (`--ok` ring, mono command,
  "fresh container · run branch"), Condition (2px accent ring, mono expression), human Gate, stop
  Gate (`--bad`, "max_iterations reached"), Goal met. Edge labels are small pills on `--bg` with
  tinted hairlines (`pass`, `fail · reset, fresh run, same session`, `approved`, `iteration >= 8`).
  Bottom-left: a zoom pill (72%) and "No model in the orchestration path — the graph decides".
- **Inspector**: node header (`Condition`, mono `node c-3`); an expression builder of 26px mono
  token fields with an accent `and` joiner and a ghost "add clause"; a compiled-expression card with
  `--ok` hairline ("total · evaluable in the core · reproducible from stored events"); `OPERANDS —
  EVERY ONE IS A COLUMN` chips (active ones accent-tinted) with "No code, no model, no I/O —
  anything this cannot express is a Gate."; then `GUARDRAILS`: max_iterations stepper (8), a
  toggle (reflection before retry, on = accent), kill & reassign stepper (3), idle detection 60s,
  wall-clock ceiling none, token budget none, and the budget/accounting + "Waiting ≠ idle" note.
  Footer: "Validate graph" / "Save workflow".

### 13. Workshop · Harnesses
Header: "Registered harnesses 12" + "Mechanism lives in the file; policy lives here. Every harness
has every capability — only the mechanism differs."; a legend (accent = native, `rgba(212,97,79,.55)`
= downgraded, "each names its loss") and "Register a harness". Body: **4-col card grid**, one per
harness (claude, codex, copilot, pi, omp, gemini, hermes, cursor, aider, opencode, dsh, antigravity —
matching `harnesses/*.toml`). Card head: name 13px/500, mono id, and a mechanism badge
(`hooks` accent tint, `hooks · TS ext` / `hooks · py plugin` on `#314454`, `ACP` on
`rgba(143,184,214,.18)` `#8fb8d6`). Body: injection mechanism line; a 4-row model-tier grid
(low/med/high/xhigh, mono values, high in accent, `↑ high` fallbacks in `--mu2`); and at the
bottom an **8-segment capability bar** (one per extension type; accent = native, red = downgraded)
over `8 extensions` + the downgrade count (`--bad` at 4+). Cards with heavy downgrades take a red
hairline. Footer line: "27 of 88 entries are downgrades…" and "`tui = false` is required on all 12;
a harness claiming true is refused at registration."

### 14. Wiki
Three panes: 246px tree · article · 284px sidebar.
- **Tree**: primary "Ingest a document" + "Derived, then curated — a path or a URL, not a blank
  page."; typed groups with counts — overview 1, decision 14 (`gavel`), concept 31 (`lightbulb`),
  entity 42 (`cube`, one flagged `orphan` in `--bad`), synthesis 8, source 57 (`file-text`,
  `globe`). Selected page: `--sf2` + accent ring.
- **Article**: `decision` accent tag + 15px title; metadata row with mono locator, rev, assertion
  and source counts, ingest/curate ages. Prose 13px/1.68 at 88% opacity, max 720px, mono inline
  paths; `LINKS OUT` as `[[wikilink]]` pills (`--sf` + hairline); `PROVENANCE` list with icons.
- **Sidebar**: a `GRAPH` SVG (258×132; 7px accent center node, `#0d5480` and `#314454` neighbors,
  hairline edges, 8px caption) + "Pages are nodes, wikilinks are edges — the canvas renderer,
  repointed."; a `CONTRADICTIONS` card (accent ring) with two conflicting mono values and their
  sources plus "Adjudicate" / "Board card"; and `LOCUS WIKI LINT` — orphans, broken link, unnamed
  entities, unsourced assertion, and `--ok` "153 pages otherwise clean". Footer: "The wiki is
  curated prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else."

## Interactions & Behavior
- **Rail click** → that category's first view. **Tab click** → that view. Both are instant, no
  transition. Nothing else in the mockup navigates except the Extensions `agents` card and the
  `← Extensions` back link.
- **Session select** (Automate · Agents) swaps the transcript, header and conditional footer; other
  sessions keep running.
- **Detach / minimize** on a session header: detach opens the same app in a second Tauri window in
  detached mode (never a second webview in one window); minimize sends the session to the strip tile.
- Live indicators: pulsing accent dots for running agents, blinking carets for input affordances.
  Both are pure CSS; nothing else animates.
- Hover/active/focus were not drawn. Apply the repo's shadcn-solid/Kobalte states: hover lifts a
  surface one step (`--sf` → `--sf2`), pressed goes one step further, and keyboard focus is
  `outline: 2px solid var(--ac); outline-offset: 2px` — never a browser default ring.
- Loading/empty/error states were not drawn and need design decisions: the honest defaults are
  "silence is the default" (no spinners on the inbox), skeleton rows for the tables, and a stated
  reason for every empty pane.
- Fixed 1440×900 layout, no responsive behavior. Panes are resizable in the real app
  (`src/panes`) — the mockup's widths are the defaults.

## State Management
Minimal in the mockup, and worth keeping thin in Solid signals/stores:
- `view`: one of `inbox status plan wiki develop board sessions telemetry runs artifact extensions
  agents canvas harnesses`; derives category, category label, locator, and the visible tab set.
- `selectedSession`: index into the session list; derives the transcript, header, and which
  conditional footer (stuck / waiting / none) renders.
Real data to source over Tauri commands + event subscriptions: inbox items, project/status metrics,
sessions and their event streams (typed events, not PTY, for Agent Panes), board tasks, git status
and diffs for the Develop panel, telemetry aggregates and facet counts, artifacts and comments,
extension inventory + materialization report, workflow graph, harness registry (`harnesses/*.toml`).

## Assets
None external beyond fonts and icons: **Inter** (system), **JetBrains Mono** (Google Fonts) and
**Phosphor Icons** regular + fill (CDN in the mockup — vendor both into `apps/desktop` for a desktop
app that must work offline). The mockup's SVGs (wiki graph, workflow edges, charts) are hand-drawn
in the file; rebuild them as real renderers. No images.

## Design system
The mockup was built against the **Nocturne** dark design system (dense, 8px radii, accent as line
and glow rather than fill, outlined primary buttons, no pure black or white). The visible palette is
Nocturne re-toned to Locus's amber accent (`#ffbb39`). Its `.btn`, `.tag`, `.card`, `.table`,
`.input` and `.seg` classes appear throughout — map each to its shadcn-solid equivalent in
`src/ui` rather than copying Nocturne's stylesheet in.

## Screenshots
`screenshots/` holds one capture per screen, at the design's own 1440×900 (scaled to fit the capture
viewport — read pixel values from the token tables above, not off the images):
01-inbox · 02-status · 03-plan · 04-develop · 05-automate-kanban · 06-automate-agents ·
07-review-telemetry · 08-review-runs · 09-review-artifacts · 10-workshop-extensions ·
11-workshop-agent-definitions · 12-workshop-workflow · 13-workshop-harnesses · 14-wiki.

## Files
- `Locus.dc.html` — the full mockup (all 14 screens + shell; the design reference).
- `support.js` — the mockup's runtime. Included only so the HTML opens; **not** part of the design.
- Tweakable props on the mockup: `accent` (color, default `#ffbb39`), `railLabels` (bool),
  `showStrip` (bool). Worth keeping the first as a theme token.

## Repo touchpoints
- `apps/desktop/src/App.tsx` — replace the Tauri scaffold with the shell (title bar, rail, tab bar, strip).
- `apps/desktop/src/ui/` — shadcn-solid components: button, tag/badge, card, table, input, textarea,
  segmented control, tooltip.
- `apps/desktop/src/panes/` — Agent Panes (event stream), Shell Panes (xterm.js), editor + diff pane,
  minimize-to-tile strip.
- `apps/desktop/src/workflow-canvas/` — the Workflow screen (nodes, edges, inspector).
- `harnesses/*.toml` — the source of truth for the Harnesses screen.
- `PLAN.md`, `docs/adr/` — the vocabulary the copy uses (waiting ≠ idle, clone not mount, byte-
  deterministic materialization, fixed board columns, one gate at the end of a plan).
