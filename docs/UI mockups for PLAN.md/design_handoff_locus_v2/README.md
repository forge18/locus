# Handoff: Locus desktop UI — v2

## Overview
A single high-fidelity HTML mockup of the **Locus** desktop app — the multi-project, multi-agent
orchestration client described in `PLAN.md`. It covers the whole application shell (title bar,
project-scoped rail, running-agent popover) and **31 interior screens**: Inbox, Dashboard, Projects
(Settings + Analytics), Plan (Conversation / Spec / Tasks & cards), Develop, Automate (Kanban +
Agents), Dispatch (Autorun / Schedules / Runs), Memory (Short-term / Long-term / Artifacts / Wiki),
Review (Telemetry), Settings, Mail, and Workshop (Agents, CLI, Commands, Harnesses, Hooks, Linters,
Output styles, Providers, Rules, Skills, Workflows — Visual + Governance).

Target repo: `/Users/forge18/Repos/locus` — implement inside `apps/desktop`
(**Tauri 2 + SolidJS + Vite + TypeScript**). Honor the repo's own notes: `src/ui/README.md`
(shadcn-solid over Kobalte, owned locally), `src/panes/README.md` (Agent Panes = typed event
streams, Shell Panes = real PTY via xterm.js, one webview per window), `src/workflow-canvas/README.md`.

**This supersedes `design_handoff_locus_desktop_ui/` (v1).** The v1 token table, type scale, shell
anatomy and per-screen detail for Inbox, Develop, Automate, Telemetry, Runs, Artifacts and Wiki all
still hold — read that README for those. This one documents what v2 adds and changes.

## About the design files
`Locus v2.dc.html` is a **design reference created in HTML**, not production code. Do not port its
markup: recreate the screens as SolidJS components using the repo's patterns. The mockup's runtime
(`support.js`, `<x-dc>`, `<sc-if>`, `<sc-for>`) is authoring scaffolding with no analogue in the app.
Inline styles are an artifact of the authoring environment — express the values as CSS variables.

`Locus v2 (standalone, offline).html` is the same mockup bundled into one self-contained file (fonts,
icons and design system inlined). Open it in any browser, no server, no network.

## Fidelity
**High-fidelity.** Final colors, type sizes, spacing, density and copy. Deliberately dense: a
professional tool at ~1450×930. Do not loosen the spacing.

## Design tokens (v2 palette)
Two accents, and they never do each other's job:

| Token | Value | Role |
| --- | --- | --- |
| `--ac` | `#ffbb39` | **you must act** — needs-you, selection, focus, gates, approvals |
| `--ac2` | `#9184d9` | **a machine is working** — running agents, live/pulse, machine-authored |
| `--bg` / `--bg-deep` | `#1d2731` / `#151d25` | app ground / rail, title bar, footers |
| `--sf` / `--sf2` / `--sf3` | `#22303c` / `#293947` / `#314454` | surface / raised-selected / chips |
| `--tx` / `--mu` / `--mu2` | `#eef2f6` / `.78` / `.62` white | text ramp |
| `--line` / `--line2` | `.14` / `.24` white | hairlines, borders |
| `--ok` / `--bad` | `#68ad91` / `#df8a7d` | pass / fail (solid variants `#4fa07f` / `#d4614f`) |
| `--data-1…3`, `--data-hi` | `#35495a` `#4e6c81` `#62869e` `#8fb8d6` | **magnitude ramp — all bars and charts** |
| `--blue` / `--blue-lit` | `#083c5d` / `#0d5480` | agent avatar grounds |
| `--fm` | JetBrains Mono | identifiers, paths, ids, model names, numerics |

Rules that were violated in v1 and are now enforced: **the accent is never a bar or a fill** —
magnitude uses the `--data` ramp; no more than a handful of accent-background elements per screen
(there is a linter for it on the Workshop → Linters screen). Body font Inter, 400 with 500 for
emphasis, never 600+. Selection is `inset 0 0 0 1px var(--ac)` over `--sf2`. Icons: Phosphor
regular + fill only (no bold weight is loaded). Two keyframes: `pulse` (live) and `blink` (carets).

## Shell
Window 1450×930, radius 11px. Title bar (42px): traffic lights, `LOCUS`, current category + view
label, and a right-side **running-agent pill** (`8 running` with a pulsing `--ac2` dot, `1 needs you`
in `--ac`) that opens a popover of every active session.

**Rail (212px)** — the navigation model changed in v2. It is project-scoped:
- Global, above the divider: **Inbox** (badge), **Dashboard**, **Projects**.
- A **project card** (`--ac2` tinted, rounded) holding the current project switcher (`#tapestry`,
  type-to-filter menu with match highlighting) and the four project-scoped views: **Plan**,
  **Develop**, **Automate**, **Review**. Everything inside this card is scoped to the selected project.
- Global, below: **Dispatch** (with a status dot — see below), **Memory** (expands to Short-term /
  Long-term / Artifacts / Wiki), **Settings**, **Workshop** (expands to Agents, CLI, Commands,
  Harnesses, Hooks, Linters, Output styles, Providers, Rules, Skills, Workflows).
- Foot: `⌘K` search-or-jump.

**Dispatch status dot** — green when at least one agent is running; amber when nothing runs but
autorun is armed somewhere; red when dispatch is fully stopped. It is the one always-visible
answer to "is anything happening?".

## What v2 adds

### Providers (Workshop)
New. Credentials and the short list of models actually used.
- Left rail: providers with a state dot (ok / expiring / not configured) and "n preferred".
- **Authentication**: method segmented (OAuth / API key / None), masked secret with Reveal and
  Replace, `base_url` override, and a verify line ("verified 11m ago · 327 models listed").
  Secrets live in the OS keychain; Locus stores the reference, never the key.
- **Preferred models**: a table of model id · **alias** · context · in/out price · in-selector
  toggle, plus a catalogue search ("4 of 327 match") to add more. The alias is what every model
  selector shows from then on, for every harness pointed at that provider.
- Right rail: a preview of the selector as the user will see it, which harnesses use the provider,
  and 30-day spend.

### Harnesses (Workshop) — rewritten
Routing, capture and injection are **gone**. A harness record is now:
- `identifier`, `adapter` (an adapter must exist or the harness cannot be selected anywhere),
  `providers` (chips, only providers configured under Providers), `default model`, `default effort`.
- **Autorouting** with an on/off switch. Off: every task runs on the record's default model and
  effort. On: a six-band table — `xtra-low · low · medium · high · xtra-high · max` × **model** ×
  **effort** × **approval tick** × when-to-use prose. A band with no model falls to the next band up.
  A ticked band waits for the user before it starts.
- **Adapter config**: a free-form key / value / type table with "Add config key", so later config
  lands without a schema change.
- All eight extension types are supported on every harness. What differs is mechanism, and mechanism
  is the adapter's problem — the downgrade vocabulary is gone from this screen.

### CLI (Workshop)
New. Built-in tools grouped by category (source control, search & files, Rust, database, network),
each with a per-tool toggle and a per-group master toggle showing mixed state. Then **Your own**:
uploaded tools with signature state (unsigned ⇒ read-only roles only), a drop zone for a binary /
script / `tool.toml`, and `install` + `verify it landed` fields. Side panel: base-image size and
last rebuild, plus "most reached for" call counts. Enabled tools are baked into the image once, not
installed per run.

### Projects — two views
The project detail pane is split by a segmented control next to the project name:
- **Settings** — *Harnesses* (allow-list with the adapter gate, one **agent default**, per-row
  routing summary), *Repos*, *Base context* (moved here from Workshop: token-budget meter and an
  editable `base.md`), *Extensions* (eight groups pulled from the Workshop defaults, per-item and
  per-group toggles — switching one off removes it from the materialized tree without deleting it),
  *CLI tools* (search-and-add from the enabled set, with per-role scope).
- **Analytics** — the tag counters, the agents that have run here, and the statistics table
  (days, tasks, runs, spend, per-model tokens / cache / spend).

### Plan — decompose, and editing
The pipeline is **9 stages**: Inputs → Orient → Converse → Synthesise → Audit → Recommend →
Override → **Decompose** → Approve. Three tabs, centred in the header:
- **Conversation** — as v1 (interviewer / researcher / auditor, inline scope decision, live line).
- **Spec** — `spec.md` as an editable document: stable requirement ids (`R-05`…), one requirement
  per block, the audit finding attached inline with "Mark finding resolved", an outline rail, and
  "unsaved" state. Saving re-runs stage 5 over the changed requirements only.
- **Tasks & cards** — stage 8, *Decompose*: **what becomes a kanban card**. Three choices — *The
  spec* (1 card, the agent decomposes at run time), *Every task* (one card each), *Spec + carve-outs*
  (recommended: the spec rides as one card and the long tasks get their own). Below, the task table
  is editable (title, role, estimate, dependency) with a per-row toggle between "its own card" and
  "rides on the spec card", and a footer that states the resulting card count.

### Dispatch — kill switch
A red **Stop all** in the header opens a confirm dialog that names exactly what it touches
(8 running agents, autorun in 5 projects, 3 schedules — branches, artifacts and memory untouched),
with a toggle for whether each agent writes its handoff first. Confirming stops everything, turns
autorun off everywhere, shows a stopped banner with a 10-minute **Restore previous state**, and
flips the rail dot to red.

### Settings — parallelism
New section in Guardrails: **max parallel agents** (global), **max per project**, **priority method**
(plan order / manual / unblocks-most / shortest first, each explained), **tie-break**, and
**preempt a running agent** (off: a higher-priority card waits for a slot; on: the lowest-priority
run is paused at its next iteration boundary and keeps its handoff, not its context).

### Workflows (Workshop) — list + two views
Renamed from "Workflow", and **purely an authoring surface — no run state anywhere**.
- Header: editable title on its own line, then a centred **Visual / Governance** switch, an
  autosave indicator ("saved 2s ago") and an **Inspector** show/hide toggle. No Save, no Validate.
- **All workflows** list: Published / Draft / Archived, with authoring meta (node count, last
  edited, "referenced by 1 schedule").
- **Visual** — the node canvas: palette (Agent, Task, Loop, Condition, Gate, Verify + presets),
  dot-grid canvas with an SVG edge layer, and a node inspector (expression builder, compiled
  expression, operand chips). There is **no Goal node** — the goal lives in Governance. The
  inspector ends by pointing there.
- **Governance** — **Goal** (the guiding statement, also the termination condition), **Guardrails**
  (markdown: each is a card with an editable title and a prompt body, read by the run while it is in
  flight; "Add a guardrail" gives a title field and an empty prompt box), and **Success criteria**
  (checked after the run: kind chip *command / assertion / human*, the criterion, and **checked by**
  — the core, or a gate to the user. Results land on the run, not here).

## Interactions
- Rail click → that category's first view; sub-item click → that view. Instant, no transition.
- Project switcher: type-to-filter with match highlighting; keyboard hints for move/select.
- Everything toggleable in the mockup is live state: harness allow-list and agent default,
  extension toggles, CLI tool toggles, autorouting on/off, plan granularity and per-task carve-out,
  provider selection, the kill switch, guardrail draft, panel show/hide.
- Live indicators: `--ac2` pulsing dots for running agents, blinking carets for input affordances.
  Nothing else animates.
- Hover/pressed/focus were not drawn: hover lifts a surface one step (`--sf` → `--sf2`), pressed one
  further, keyboard focus is `outline: 2px solid var(--ac); outline-offset: 2px`.
- Fixed layout, no responsive behavior. Pane widths are defaults; panes are resizable in the app.

## State in the mockup
`view` plus small per-screen state: selected session / provider / workflow, plan tab and granularity,
project tab, harness allow-list and default, extension and CLI off-lists, autorouting on/off,
dispatch stopped, panel visibility. Real data comes over Tauri commands and event subscriptions.

## Screenshots
`screenshots/` — 31 PNGs, whole window in frame, in navigation order:
`01-inbox` · `02-dashboard` · `03-project-settings` · `04-project-analytics` ·
`05-plan-conversation` · `06-plan-spec` · `07-plan-tasks-decompose` · `08-develop` ·
`09-automate-kanban` · `10-automate-agents` · `11-dispatch-autorun` · `12-dispatch-schedules` ·
`13-dispatch-runs` · `14-memory-short-term` · `15-memory-long-term` · `16-memory-artifacts` ·
`17-memory-wiki` · `18-review-telemetry` · `19-settings-guardrails` · `20-workshop-agents` ·
`21-workshop-cli` · `22-workshop-commands` · `23-workshop-harnesses` · `24-workshop-hooks` ·
`25-workshop-linters` · `26-workshop-output-styles` · `27-workshop-providers` · `28-workshop-rules` ·
`29-workshop-skills` · `30-workflows-visual` · `31-workflows-governance`.
Read pixel values from the token tables, not off the images (they are scaled to fit the capture).

## Files
- `Locus v2.dc.html` — the mockup (all screens + shell).
- `Locus v2 (standalone, offline).html` — same thing, self-contained.
- `support.js` — the mockup's runtime. Included only so the HTML opens; not part of the design.
- `_ds/nocturne-…/` — the Nocturne design system the mockup was built against (`styles.css`,
  bundle, guide). Map its `.btn` / `.tag` / `.card` / `.table` / `.input` / `.seg` classes to
  shadcn-solid equivalents in `src/ui` rather than copying the stylesheet in.
- Tweakable props: `accent`, `accent2`, `toasts`, `terminal`.

## Repo touchpoints
- `apps/desktop/src/App.tsx` — shell: title bar, project-scoped rail, running-agent popover.
- `apps/desktop/src/ui/` — button, tag, card, table, input, textarea, segmented control, toggle.
- `apps/desktop/src/panes/` — Agent Panes (event stream), Shell Panes (xterm.js), editor/diff pane.
- `apps/desktop/src/workflow-canvas/` — the Workflows Visual view.
- `harnesses/*.toml` — source of truth for Workshop → Harnesses (adapter presence gates selection).
- New surfaces with no repo home yet: providers + keychain broker, CLI tool registry and image
  build, plan decomposition (spec ↔ card mapping), dispatch queue with the priority method and cap.
- `PLAN.md`, `docs/adr/` — the vocabulary the copy uses. Note one deliberate departure: PLAN.md says
  Locus holds no model API keys; **Providers introduces exactly that**, and the ADR needs updating
  or the screen needs revisiting.
