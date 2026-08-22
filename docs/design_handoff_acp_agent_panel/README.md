# Handoff: ACP Agent Panel (Locus)

## Overview

The **agent panel** is the end-user surface for an ACP (Agent Client Protocol) session inside Locus. It answers two questions at once:

1. **What is the agent doing?** — a streamed transcript of thinking, tool calls, file edits, terminal output and citations, plus a persistent plan.
2. **How do I steer it?** — a composer with slash commands and @-mentions, permission gates that block on the user, elicitation forms, checkpoints to roll the workspace back, and a research pane that survives the run.

It is **one view**, not two. Everything the user needs to supervise a run lives in a single scrolling column with a persistent header, a plan dock, and a composer; the research pane opens as a second column on the right. The design intentionally avoids a separate "monitor" screen — supervising and steering are the same activity.

## About the Design Files

The file in this bundle is a **design reference created in HTML** — a working prototype that shows intended look and behavior. **It is not production code to copy.** `Agent Panel.dc.html` is a "Design Component": a single HTML file with an inline-styled template plus a small logic class, rendered by the bundled `support.js` runtime. That runtime is a prototyping harness; do not port it.

The task is to **recreate this design in the target codebase's existing environment** (React/TypeScript, Tauri webview, or whatever the app already uses), following its established component patterns, state management, and styling approach. Where the prototype fakes data with hardcoded strings, wire the real ACP session stream instead.

If the target app has no UI environment yet, choose the framework that fits the rest of the stack and implement there.

### How to open the prototype

Open `Agent Panel.dc.html` in a browser (it loads `support.js` from the same folder, plus Phosphor Icons and Google Fonts from CDNs). Resize the window to see the responsive collapses. The prototype has interactive tweaks (see **Prop matrix** below) that switch between run states; in the browser they use their defaults, so to see other states change the default in the `data-props` JSON on the `<script data-dc-script>` tag or read the state table below.

## Fidelity

**High-fidelity.** Colors, typography, spacing, radii, icon choices, hover states and interaction behavior are final and intended to be matched. The layout is production-intent, not a sketch.

Two things are deliberately *not* final:
- **Copy in the transcript** is representative sample content (an auth-token-rotation refactor). Real content comes from the session stream.
- **Avatar treatment** (the `RF` monogram tile) is a placeholder for whatever agent-identity artwork the product settles on.

## Design language

The panel uses the project's **Nocturne** dark theme — a deep desaturated blue-slate ground, amber as the single action accent, violet as the "machine activity" accent, and monospace for every machine-authored string (paths, ids, numbers, timestamps, tool names).

The governing rules, in priority order:

1. **The stream is the product.** Chrome is quiet — 44px header, 1px rules, no card shadows in the transcript. Emphasis is spent on the agent's output.
2. **Machine strings are monospace.** Paths, hashes, token counts, timestamps, model names, workflow names, tool names → JetBrains Mono. Prose and labels → Inter.
3. **Amber means "you".** Anything requiring or recording a human decision is amber: permission gates, elicitation, checkpoints, the primary button, the caret. Violet means "the machine is working": thinking blocks, activity, live counters.
4. **Nothing floats.** No drop shadows in the transcript; separation comes from 1px rules, inset ring `box-shadow`s, and surface steps (`--sf` → `--sf2` → `--sf3`).
5. **Blockers dock, they don't overlay.** A permission request or elicitation pins to the bottom of the stream above the composer, dims the transcript behind it, and can be minimized to a one-line pill. It never becomes a modal — the user can always read the transcript and type.

## Layout

```
┌──────────────────────────────────────────────┬─────────────────┐
│ header 44px  #project │ T-118 name  ⟨workflow⟩ ⟨Research⟩ ⋮   │
├──────────────────────────────────────────────┤   research      │
│ ⟨live pill⟩                                  │   pane          │
│                                              │   380px         │
│   stream (scrolls)                           │   (toggle)      │
│     user bubble (right, 74% max)             │                 │
│     agent turn (left, full width)            │                 │
│       thinking · text · tool cards · diffs   │                 │
│       checkpoint markers                     │                 │
│                                              │                 │
├──────────────────────────────────────────────┤                 │
│ ⟨docked blocker: gate or elicitation⟩        │                 │
├──────────────────────────────────────────────┤                 │
│ plan dock (collapsed 34px / expanded list)   │                 │
├──────────────────────────────────────────────┤                 │
│ composer (textarea + send)                   │                 │
│ toolbar: agent · model · effort │ Manual/Auto │ context meter  │
└──────────────────────────────────────────────┴─────────────────┘
```

Root: `display:flex; height:100vh; overflow:hidden`. Left column `flex:1; min-width:0; display:flex; flex-direction:column; position:relative`. The stream is the only `flex:1` child; everything else is `flex:none`, so docked elements steal height from the transcript and never push the composer off-screen.

**Vertical budget rule (important).** Docked blockers use `flex:0 1 auto` with `max-height` caps (gate 46%, elicitation 52%) and internal `overflow:auto`. When both are docked at once they stack and the transcript shrinks — the composer and plan dock always remain visible. Do not give docked elements a fixed height.

---

## Regions in detail

### 1. Header (44px)

`height:44px; padding:0 11px; background:rgba(0,0,0,.3); border-bottom:1px solid var(--line); z-index:40`

Left to right:

| Element | Spec |
|---|---|
| Project handle | `#tapestry` — mono 13px, `--ac2` violet, ellipsizes first |
| Divider | 1px × 18px, `--line` |
| Task chip | `T-118` — mono 11px `--mu` on `--sf2`, radius 5px, padding 1px 6px, Phosphor `ph-kanban` 11px. Hidden when the session is not linked to a task |
| Session name | Inter 14px `--tx`, single line, ellipsis. Click to rename: the box gets an amber inset ring, a 1px × 15px amber caret blinks (1.1s), and the trailing icon swaps `ph-pencil-simple` → `ph-check` |
| Workflow chip | right-aligned group, immediately left of Research: `ph-flow-arrow` 12px in `--ac2` + workflow name in mono 11.5px `--mu`, on `--sf` with a `--line` inset ring, radius 6px. **Only rendered when a workflow is assigned to the session** |
| Context total | mono 12px, `38.4k tok · $0.41` — only when cost display is on |
| Research toggle | `ph-flask` + "Research" + count badge. Active: `--ac2-dim` fill, `--ac2` text, violet inset ring. Inactive: `--mu` on transparent |
| Overflow | `ph-dots-three-vertical` 17px `--mu2` |

### 2. Live status pill (persistent, floating)

`position:absolute; top:11px; left:16px; z-index:40` — sits above the stream *and* above the blocker scrim, so it is always readable.

`display:inline-flex; gap:7px; padding:4px 10px; border-radius:20px; font:400 11.5px Inter; color:var(--tx); box-shadow:0 2px 10px rgba(0,0,0,.45)` with a **solid** background per state (deliberately opaque so it stays legible over scrolling content):

| State | Background | Label | Dot |
|---|---|---|---|
| working | `#4b4183` | `working` | `--ac` amber, `animation: breathe 1.4s ease-in-out infinite` (opacity 1 → .2) |
| waiting on you | `#7a5410` | `waiting on you` | `--ac` amber, static |
| idle | `#314454` | `idle` | `--ac` amber, static |

The breathing dot is the liveness signal — it tells the user the session is not frozen. It stops when the agent is not actually progressing. Clicking the pill scrolls/flashes the thing that needs attention.

**The state is derived, not set.** `waiting on you` is true whenever a permission gate is pending *or* an elicitation is open; `working` is "not idle and nothing blocking". Do not model `requires_action` as an independent toggle — it is what the agent reports when it raises a request.

### 3. Stream

`flex:1; min-height:0; overflow:auto; padding:14px 16px 18px`. Wrapped in a `position:relative` container that also holds the blocker scrim.

#### User bubble

Right-aligned, `max-width:74%`, `padding:9px 12px`, radius 10px, background `--blue` `#083c5d` with a `--blue-lit` inset ring.

- **Header, inside the bubble:** avatar tile (22px, radius 6px, `--sf` with `--line2` ring, `ph-user` 12px) + `You` (Inter 500 12.5px) on the left; `ph-copy` 13px + time (mono 11.5px `--mu2`) pushed right.
- **Body:** Inter 14.5px, line-height 1.55. Collapsed to 2 lines by default with a trailing caret; click to expand.
- **Footer, outside the bubble:** `You · 09:14` — 11.5px `--mu2`, name in Inter, time in mono.

#### Agent turn

Full width, left-aligned. Container: `padding:11px 13px; border-radius:10px; background:var(--sf); box-shadow:inset 0 0 0 1px var(--line)` — plus, when the turn is live, an amber/violet left edge (`inset 3px 0 0`) that tracks the run state.

- **Header, inside the container:** `RF` monogram tile (24px, radius 7px, `--blue-lit` fill, `--blue` ring, mono 500 10px, `#bfe0f5`) + `refac` (Inter 500 13.5px) + `refactor` (12px `--mu2`) on the left; `ph-copy` + time on the right.
- **Footer, outside the container:** `refac · 09:16` — 11.5px `--mu2`; appends `6.2k in · 1.4k out · $0.09` in mono when cost display is on.

Both bubbles use the **same** header/footer structure — avatar + name left, copy + time right, inside; name · time, outside. Only alignment differs.

#### Content families inside a turn

Five families, each with a distinct visual treatment. This vocabulary is the core of the design — keep it.

**a. Thinking block** — collapsible, `background:rgba(0,0,0,.22)`, radius 8px, `--line` ring. Header: `ph-eye` 13px `--ac2` + `Thought for 8s` (Inter 500 12.5px) + `· 3 considerations` (`--mu2`) + a 3-way segmented control on the right: `Summary | Full | Hidden` (11px, active = `--sf3` fill + `--tx`, inactive `--mu2`). Body: 13px `--mu`, line-height 1.6. In `Full`, each consideration is a row with a mono index. In `Hidden`, the block collapses to the header only.

**b. Prose** — Inter 14.5px, line-height 1.68, `--tx`, `max-width:70ch`. Inline file paths are mono 13px `--data-hi` with a dotted underline; they are clickable (open in an editor pane).

**c. Tool call card** — radius 9px, `--line` ring, `background:rgba(0,0,0,.16)`. Header row (`padding:7px 10px`, `background:var(--sf2)`, `border-bottom:1px solid var(--line)`): status icon + tool name (mono 12.5px `--tx`) + target (mono 12px `--mu`) + right-aligned duration (mono 11.5px `--mu2`) and a caret. Status icons: running `ph-circle-notch` `--ac2` with a 1s spin, ok `ph-check-circle` `--ok`, failed `ph-x-circle` `--bad`, queued `ph-clock` `--mu2`. Body varies by tool:
  - **read/search** — a compact result list, mono 12.5px, one row per hit, path + line number, `--mu`.
  - **terminal** — `background:rgba(0,0,0,.34)`, mono 12.5px, line-height 1.62, per-line colors (`--ok` pass, `--bad` fail, `--mu2` chrome). Prefix `$` for the command in `--data-hi`.
  - **file edit** — a diff (below).

**d. Diff** — mono 12.5px, line-height 1.58. Hunk header `@@ -42,9 +42,7 @@ impl Session` in `--mu2` on `rgba(0,0,0,.24)`. Rows: gutter (mono 11.5px `--mu2`, 30px, right-aligned), sign, code. Removed: `background:rgba(212,97,79,.13)`, text `--bad`. Added: `background:rgba(79,160,127,.13)`, text `--ok`. Context: `--mu`. Horizontal `overflow:auto`, never wraps.

**e. Citation / research chip** — inline `ph-link` 11px + source label in mono 11.5px `--mu2` on `--sf2`, radius 5px. Clicking pins it into the research pane.

#### Checkpoint markers

A checkpoint is a **timeline marker in the stream**, not a footer on a card. One row, `display:flex; align-items:center; gap:10px; margin:-4px 0 14px`:

`⟨pill⟩  before auth/token.rs · 09:28  ┈┈┈┈┈┈┈┈┈┈┈┈┈┈  Restore`

- **Pill:** `ph-clock-counter-clockwise` 11px + `Checkpoint 7`, Inter 11.5px, radius 20px, padding 2px 9px. Idle: `--sf2` fill, `--mu` text, `--line2` inset ring. Restored: `rgba(255,187,57,.16)` fill, `--ac` text, `rgba(255,187,57,.5)` ring.
- **Label:** 11.5px `--mu2`, `white-space:nowrap`.
- **Rule:** `flex:1; border-top:1px dashed var(--line)`.
- **Action:** `Restore` 11.5px `--ac`; becomes `restored` in `--mu2` once used.

Naming: checkpoints are **`Checkpoint N`** in Inter — sequential and speakable ("restore checkpoint 7"), *not* an opaque id like `ck-07`. The identity/hash, if the backend needs one, stays out of the UI.

**Restored state** — an amber banner drops in below the marker: `padding:7px 11px; border-radius:7px; background:rgba(255,187,57,.06); box-shadow:inset 3px 0 0 var(--ac)`, with `ph-arrow-counter-clockwise` `--ac`, the sentence *"Workspace reverted to Checkpoint 7 — 2 files, 13 lines. The transcript is kept."*, and an `Undo` at the right. The transcript is never truncated by a restore — that distinction matters and the copy states it.

### 4. Docked blockers

Two kinds, same mechanics. Both `flex:0 1 auto; margin:0 16px 12px; border-radius:9px; background:var(--sf); box-shadow:inset 0 0 0 1px rgba(255,187,57,.45), inset 3px 0 0 var(--ac)`, internal `overflow:auto`.

**Scrim.** While a blocker is docked, an overlay covers the stream (and only the stream): `position:absolute; inset:0; z-index:30; background:rgba(8,12,16,.82)`. The plan dock, composer and live pill stay above it. Clicking the scrim minimizes the blocker.

**Minimize.** A `ph-minus` 13px `--mu2` in the card header collapses the card to a **one-line pill** in the same dock slot: `padding:8px 11px; radius:7px; background:rgba(255,187,57,.09); border:1px solid rgba(255,187,57,.45); box-shadow:inset 3px 0 0 var(--ac)` — icon + `NEEDS YOU · EDIT` (Inter 500 10.5px, `letter-spacing:.13em`, uppercase, `--ac`) + the target + `Review` / `Answer` at the right. The scrim clears and the plan re-expands. Clicking the pill restores the card and the scrim.

#### a. Permission gate

Header: `ph-seal-warning` (fill) 15px `--ac` + `NEEDS YOU · EDIT` label + full path (mono 12.5px `--data-hi`, dotted underline, ellipsized from the left) + `+2 −3` counts (mono 12px, `--ok`/`--bad`) + time + minimize.
Body: the diff, `max-height:46%`, scrollable.
Actions row (`padding:10px 12px; border-top:1px solid var(--line)`): **Approve** — amber solid `--ac`, `#1d2731` text, Inter 500 13px, radius 7px, `ph-check` 13px; **Decline** — `--sf3` fill, `--tx` text; then a checkbox + `Approve the remaining 2 edits in this turn` (12.5px `--mu`), which is how the user escapes per-edit prompting without switching to bypass mode.

On resolve, the card is replaced in the transcript by a resolved edit card (`ph-check-circle` `--ok`, `approved · applied`), followed by its checkpoint marker.

#### b. Elicitation

Header: `ph-list-dashes` 15px `--ac` + `NEEDS YOU · ELICITATION` + `refac is waiting on an answer` + time + minimize.
Body (`max-height:52%`): the question in Inter 14.5px `--tx`, then the schema rendered as native-feeling controls — radio rows (18px circle, `--line2` ring, amber 8px dot when selected, label 13.5px, optional 12px `--mu2` helper), text inputs (`--bg-deep` fill, `--line2` ring, radius 7px, Inter 13.5px), and a free-text "something else" field.
Actions: **Send answer** (amber solid) + `Skip` (ghost, `--mu2`).

Answered state collapses to a compact record in the transcript: the question in `--mu2` and the chosen answer in `--tx`, with an `Edit` affordance.

### 5. Plan dock

`flex:none; border-top:1px solid var(--line); background:rgba(0,0,0,.22)`.

**Collapsed (34px):** `ph-list-checks` 14px `--mu2` + `Plan` (Inter 500 12.5px) + a tick strip (5 × 14px×3px bars, radius 2px: done `--ac2`, current `--ac`, pending `--line2`) + `2 / 5` (mono 12px) + a status dot + the current step title (13px `--tx`, ellipsized) + step meta (mono 11.5px `--mu2`: `step 3 · waiting on you` / `step 3 · in flight` / `step 5 of 5`) + a caret.

**Expanded:** the collapsed row stays as the footer; above it, one row per step (`padding:6px 4px`): status icon (`ph-check-circle` `--ok` done, `ph-circle-notch` spinning `--ac2` live, `ph-minus-circle` `--mu2` pending), mono index, title (13.5px — `--tx` for the live step, `--mu` otherwise), right-aligned outcome (12px `--mu2`: `3 found`, `staging, 24h`, `4 tests`). A footer line reads `1 draft kept`.

**Auto-collapse:** the plan collapses itself whenever a blocker is docked, and re-expands when the blocker is minimized or resolved. Two competing bottom-docked panels at once is the failure mode this avoids.

### 6. Composer

`flex:none; padding:0 16px 12px`.

Input row: `background:var(--sf); border-radius:10px; box-shadow:inset 0 0 0 1px var(--line2)` (amber ring on focus), `padding:9px 11px`. A leading `/` badge (mono 12px `--ac` on `--sf2`, radius 5px) hints the command palette. Textarea: transparent, Inter 14.5px, `--tx`, placeholder `Ask, steer, or press / for commands` in `--mu2`. Trailing send button: 30px square, radius 8px — amber `--ac` with `#1d2731` glyph when idle; `--bad-solid` with a `ph-stop` glyph while the agent is running (the stop control lives where the send control was, so it is always in the same place).

Toolbar row below (`gap:8px; margin-top:8px`, wraps):
- **Agent/model group** — one control, `--sf` fill, `--line` ring, radius 8px: `ph-sparkle` + `Claude Code` (Inter 12.5px) · `Opus 4.6` (mono 12px `--tx`) · `High` effort (mono 12px `--mu`) + caret. Opening it shows a menu of installed agents (name, transport, `ph-check` on the active one).
- **Permission mode** — `Manual | Auto` segmented control (11.5px; active = amber fill `rgba(255,187,57,.16)` + `--ac` text + amber ring). Only rendered for gated runs. Manual stops on every edit; Auto approves them as they stream.
- **Context meter** — `--sf` pill: a 54px×5px track (radius 3px) filled `--ac2` violet to the used fraction, then `12k / 200k` in mono 12px, then `ph-arrows-in-simple` to compact. Turns amber past 80%.

### 7. Research pane (right column, 380px)

`flex:none; width:380px; border-left:1px solid var(--line); background:var(--bg-deep)`. Toggled from the header.

Header: `ph-flask` 15px `--ac2` + `Research` (Inter 500 14px) + `this session` (12px `--mu2`) + `ph-x` close.
Sub-header: `seeded` chip + `2 of 4 came from the plan` (12.5px `--mu2`).

Cards (`padding:10px 11px; border-radius:9px; background:var(--sf); box-shadow:inset 0 0 0 1px var(--line); margin-bottom:9px`):
- Provenance chip — `seed` / `this run` / `at close` (mono 10.5px, `--sf2` fill; `this run` uses `--ac2-dim` + `--ac2`).
- Source (mono 11.5px `--mu2`) — e.g. `postgresql.org/docs · NOTIFY`, `tapestry · adr/0031-auth-ttl.md`, `rfc 9068 §2.2`.
- Claim (Inter 14px `--tx`, line-height 1.45) — the finding, written as a sentence.
- Body (13px `--mu`, line-height 1.5).
- Footer: `ph-link` + `claim → source` (12px `--ac`) and a timestamp.

Bottom strip: `at close` + `refac nominates 3 for long-term memory` — the handoff from session research to durable memory.

---

## Interactions & behavior

| Trigger | Result |
|---|---|
| Click session name | Inline rename; blinking amber caret; icon → `ph-check` |
| Click user bubble | Expand/collapse the prompt (2-line clamp ↔ full) |
| Thinking `Summary/Full/Hidden` | Swap the thinking body; `Hidden` collapses to the header |
| Click tool card header | Expand/collapse the card body |
| Click a file path | Opens an editor pane beside the panel (out of scope here) |
| **Approve** | Gate card is replaced by a resolved edit card + checkpoint marker; scrim clears; plan re-expands; state → working |
| **Decline** | Gate resolves as declined; the agent is told; state → working |
| Check "approve the remaining 2" | Subsequent edits in this turn auto-apply; the Manual/Auto control flips to Auto |
| **Send answer** / **Skip** | Elicitation collapses to its answered record; scrim clears |
| Minimize (`−`) or click scrim | Blocker → one-line pill; scrim clears; plan re-expands |
| Click the pill | Blocker restored; scrim returns; plan collapses |
| **Restore** on a checkpoint | Pill → amber `restored`; amber banner appears with an `Undo` |
| Click the live pill | Scrolls to and flashes whatever needs attention |
| `/` in the composer | Command palette (`/plan`, `/checkpoint`, `/mode`, `/research`, `/handoff`) |
| `@` in the composer | Mention picker (files, symbols, tasks) |
| Send while running | Queued and delivered at the next turn boundary |
| Stop button | Cancels the current turn; the transcript keeps everything already streamed |
| Toggle Research | 380px column opens/closes; the transcript reflows |
| Toggle Manual/Auto | Changes gating for subsequent edits only |

**Animations.** Only three, all subtle: `breathe` (1.4s, opacity 1→.2) on the live dot; `spin` (1s linear) on running spinners; `blink` (1.1s) on the rename caret. No entrance animations on stream content — it arrives fast and motion would fight readability.

**Responsive.** Below ~1100px the research pane closes. Below ~760px the composer toolbar wraps to two rows and the model group drops its effort label. The session name is always the first thing to ellipsize. The panel is designed to work down to ~520px wide (side-panel use).

## State

| Name | Values | Notes |
|---|---|---|
| `sessionState` | `running` \| `idle` | `waiting on you` is **derived**, never set |
| `permissionMode` | `gated` \| `bypass` | Bypass hides the gate and the Manual/Auto control |
| `permissionGate` | `pending` \| `approved` \| `declined` | Per-request; `pending` docks the diff |
| `elicitation` | `answered` \| `pending` | `pending` docks the form |
| `workflow` | string (may be empty) | Empty → no workflow chip |
| `taskLinked` | boolean | Controls the `T-118` chip |
| `thinkingDisplay` | `summary` \| `full` \| `hidden` | User-level disclosure preference |
| `showCost` | boolean | Header total + per-turn footer figures |
| `researchOpen` | boolean | Right column |
| `planOpen` | boolean | Overridden to collapsed while a blocker is docked |
| `gateMinimized`, `elicitMinimized` | boolean | Reset whenever the underlying request changes |
| `restored[checkpointId]` | boolean | Drives the amber marker + banner |

Local UI state (expanded cards, rename mode, menu open) should be transient and reset on session switch. Note one bug worth avoiding, hit during the prototype: local click-state must not permanently shadow the incoming session state — reset local overrides when the upstream value changes.

## Design tokens

Nocturne theme (from `_ds/nocturne-.../styles.css`; the prototype re-declares them in `:root` so the file is self-contained):

```css
--bg:#1d2731;  --bg-deep:#151d25;
--sf:#22303c;  --sf2:#293947;  --sf3:#314454;
--blue:#083c5d;  --blue-lit:#0d5480;
--ac:#ffbb39;                        /* amber — human decisions, primary action */
--ac2:#9184d9;  --ac2-dim:rgba(145,132,217,.16);   /* violet — machine activity */
--tx:#eef2f6;  --mu:rgba(238,242,246,.78);  --mu2:rgba(238,242,246,.62);
--line:rgba(238,242,246,.14);  --line2:rgba(238,242,246,.24);
--ok:#68ad91;  --bad:#df8a7d;  --ok-solid:#4fa07f;  --bad-solid:#d4614f;
--data-1:#35495a; --data-2:#4e6c81; --data-3:#62869e; --data-hi:#8fb8d6;
--fm:'JetBrains Mono',ui-monospace,SFMono-Regular,Menlo,monospace;
```

Solid state-pill fills (opaque on purpose): working `#4b4183`, waiting `#7a5410`, idle `#314454`.

**Type.** Inter 400/500 for UI and prose; JetBrains Mono 400/500 for machine strings. Sizes in use: 10.5px (uppercase labels, `letter-spacing:.13em`), 11px–11.5px (meta, chips), 12px–12.5px (secondary, tool headers, mono body), 13px–13.5px (thinking body, plan rows), 14px–14.5px (prose, composer, user text), 15px (root). Line-heights: 1.45 research claims, 1.55 user text, 1.6 thinking, 1.68 prose, 1.58–1.62 code.

**Spacing.** 4px base. Common: 7px chip padding-x, 9px gaps within a row, 10px–13px card padding, 12px dock margin-bottom, 14px–18px stream padding, 16px stream padding-x.

**Radii.** 5px small chips · 6px avatar tiles/chips · 7px pills, inputs, buttons · 8px thinking block, toolbar controls · 9px cards, docked blockers · 10px bubbles, composer · 20px status/checkpoint pills. No fully-rounded rectangles.

**Elevation.** Inset rings (`box-shadow: inset 0 0 0 1px …`) instead of drop shadows everywhere in the transcript. Two exceptions with real shadows: the floating live pill (`0 2px 10px rgba(0,0,0,.45)`) and menus/popovers.

**Focus.** `:focus-visible { outline:2px solid var(--ac); outline-offset:2px }`. Selection: `rgba(255,187,57,.28)`.

## Assets

- **Icons:** [Phosphor Icons](https://phosphoricons.com) v2.1.1, regular + fill weights, loaded from unpkg. Used: `kanban, pencil-simple, check, flask, dots-three-vertical, eye, circle-notch, check-circle, x-circle, clock, seal-warning (fill), list-dashes, list-checks, minus, minus-circle, arrow-counter-clockwise, clock-counter-clockwise, flow-arrow, user, copy, link, sparkle, arrows-in-simple, stop, paper-plane-right, caret-down, caret-up, x`.
- **Fonts:** Inter (400, 500) and JetBrains Mono (400, 500) from Google Fonts.
- **Design system:** `_ds/nocturne-4f4ea656-e13f-4b48-8c4f-532a78ad2114/` — the project's Nocturne theme (`styles.css` tokens + `_ds_bundle.js` components). Prefer the codebase's own components; use this only as the token source of truth.
- No raster assets. The `RF` monogram tile is CSS + text and is a placeholder for real agent identity artwork.

## Files in this bundle

| File | What it is |
|---|---|
| `Agent Panel.dc.html` | The design reference — full panel, all states |
| `support.js` | Prototyping runtime required to open the HTML. **Do not port** |
| `_ds/nocturne-…/styles.css` | Nocturne design tokens |
| `_ds/nocturne-…/_ds_bundle.js` | Nocturne component bundle |
| `README.md` | This document |

## Open questions for implementation

1. **Diff rendering at scale** — the prototype shows a single small hunk. Real edits need virtualized diffs and a collapse-large-hunks rule.
2. **Multiple simultaneous requests** — the design handles one gate + one elicitation stacked. A queue of five pending permissions needs a "3 more waiting" affordance that does not exist yet.
3. **Checkpoint retention** — how many are kept, and whether restoring one prunes the ones after it (the copy currently promises the transcript survives).
4. **Workflow provenance** — the chip names the workflow but does not link to its definition or show which node is executing. Worth deciding whether the plan dock should carry node position.
5. **Agent identity** — monogram, color assignment, and whether multiple agents can appear in one transcript.
