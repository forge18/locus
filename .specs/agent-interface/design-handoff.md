# Handoff — Locus agent panel (interior surface)

## Purpose of this brief

This is a **requirements + research transfer** for designing the **Locus agent panel** — the primary
surface a user interacts with for a single running agent session. It is NOT a design; the visual and
layout decisions are yours. Your deliverable is a high-fidelity HTML mockup of the agent panel that
matches the depth and conventions of the existing `docs/design_handoff_locus_desktop/` handoff, using the
same design tokens and shell vocabulary. This document gives you the requirements, the surfaces, the states, the copy vocabulary, and the
research that justifies each decision — so you design it, not me.

**Background.** Locus is a multi-agent, multi-harness IDE (Tauri 2 + SolidJS + TypeScript + Rust
core). "Harness" = the agent backend (Claude Code, Codex, pi, etc.). A **session** runs exactly ONE
agent over **ACP** (Agent Client Protocol) — the wire protocol. The agent panel is one of three pane
types (Agent Pane = typed event stream, Editor Pane = CodeMirror). This handoff covers the **agent
panel only**, as it appears in the pane manager, running alongside an editor pane.

Read, before designing: `docs/design_handoff_locus_desktop/README.md` (the tokens, shell, and interaction
conventions the whole app shares), `docs/design_handoff_locus_desktop_ui/README.md` (v1, for the
screens desktop does not supersede), `src/panes/README.md`, `src/ui/README.md`.

## The one principle everything else serves

**Transparent by default, everything settings-driven. Never hide information; collapse it in a
user-overridable manner.** The user can always raise the detail level; nothing is purposefully
withheld. Research-minded defaults lean conservative (see §Design language) so the initial state does
not drown the user, but no surface is a wall.

## Scope: what to mock up

A single agent panel in one of its **two permission modes** (see §Permission mode), plus, to show off
secondary states:

1. Idle state with the toolbar and one completed turn.
2. The message stream showing the **five distinct object types** (below).
3. One **permission-mode approval** (diff preview) and one **bypass-mode tool call**.
4. The **research pane** opened as a side panel.
5. The **plan** step list in the panel.

Use a concrete persona: an agent `refac` (role: refactor) working on project `tapestry`, reducing
auth churn. You can vary model/effort/harness/time.

## Object types in the message stream (design NON-negotiable)

The panel must make the following **visually and spatially distinct**. They are different content
types that must not read as one continuous scroll.

| Object | From | Rows as | Must be clear at a glance |
| --- | --- | --- | --- |
| **user input** | user | a compact card, collapsed by default, click to expand/edit/resubmit | "I said this" |
| **agent output** | agent | markdown, prose, code blocks, tables | "the answer" |
| **thinking** | agent | a summarized, collapsible block (`agent_thought`) | "what it considered, briefly" |
| **tool call** | agent | live call with kind / title / status / streaming content | "what it did" |
| **diff / plan / elicitation** | agent forces user action | inline approval / plan list / form | "what it needs from me" |

Distinct **means** distinct. Do not let thinking blend into output; do not let tool calls sit as
links in prose. Each is a recognisable card/block family.

In addition: **every rendered file path is clickable** and opens that file in a new editor pane
beside the agent panel (see §File links → editor pane). **Token/cost** may appear per-message
(settings - on/off).

## The toolbar (one row)

Left → right:

- **Identity** — agent avatar (ground uses `--blue` ramp), then `Agent@Model` then the project name.
- **Session name** — task/agent-driven title; editable inline. Sessions are not named by timestamp.
- **Harness ▾ · Model ▾ · Effort ▾** — three dropdowns. The harness drop sets *which models exist*
  for the next two. Effort is a selector over the harness's `effort` tiers.
- **Context chip** — e.g. `12k / 200k` + a compact/expand icon. Clicking expands a **context view**
  (what is in the context window / memory). The icon compacts the visual. This is the "context
  visual icon with ability to compact" control.
- **Token / cost** — a field dedicated to this that is user-settable off (default **off**). When on,
  shows a single session aggregate. Not a succinct status-line; there is no configurable footer.
- **Research** → toggles the right research panel.
- **⋮** → new session · compact · clear context · (checkpoints live in the stream, see §message
  stream).

No configurable **status line** (Claude Code's `statusLine` is rejected). These are discrete
controls, not a footer.

## Permission mode (the diff gate) — this is a headline differentiator

The run has exactly two permission postures, set at dispatch (see decision surface below):

| mode | Tool calls and edits | Prompt shown? |
| --- | --- | --- |
| **bypass** (A — `allowlist+container-enforced`) | all | never |
| **gated** (B — permission mode) | all (declined) or asks-first | yes, on irreversible/relevant actions |

In **gated** mode, when an agent proposes a file edit, the panel shows a **inline diff block**
directly in the stream — the proposed change (old → new), the file path (clickable), and **Approve /
Decline** (with optional "approve all remaining in this turn"). This is the primary surface where
permission is surfaced to the user. It is NOT a separate review pane; the approval IS the diff.

In **bypass** mode, there is no prompt at all — the run proceeds under the container-enforced
declare. The mock should show both so the absence of prompts in bypass is legible.

**A checkpoint / undo** is married to this: after each edit the panel offers a **restore** control
that returns the workspace to the pre-edit state (git + run clones make this cheap).

## Research pane (per-session feed) — second headline

A **session-scoped research feed** lives beside the agent panel, opened by the toolbar's research
button (not a persistent split). It shows:

- The findings for **this session** — sources + summaries, from the agent's research, entry per
  finding.
- Each finding is **linkable** (a claim → the source it came from).
- It **inherits** the research gathered by the session's originating **planning** of the task
  (seed).
- It can **promote** surviving findings to long-term memory at session close (a review step, not
  automatic).

It is NOT the project wiki; not project-wide. The wiki is the curated project-level surfaces.

## Slash commands & context view

- The input field supports slash commands, discoverable: **`/new`** (new session),
  **`/compact`** (compact context), **`/clear`** (clear context), and a **view context** affordance
  (see the toolbar context chip). Present them as a command palette style dropdown in the input.
- The context chip opens a **context view** — a breakdown of what is in the window (the memory
  catalog, returned tool docs, injected context).

## The research always-cli floor

Independent of the bypass toggle, the research mechanisms (the research pane and any research in the
planning module) always have a **fixed, always-installed set of CLIs** available. This is not a
per-run allowlist; it is the research surface's own stable tool floor. Exact list TBD; the guarantee
is what binds.

## Elicitation (forms)

The agent can ask the user for structured input. Per the protocol, shown as a **form** (restricted
JSON schema — flat object, primitive fields, enums) or a **URL** (for sensitive things, e.g. OAuth —
the credential never touches the panel/container; the panel shows a "You'll open a URL in your
browser →" consent row). The form must:

- Validate input **before** sending.
- Let the user **review / edit** before submit.
- Offer history cache + suggestions.
- Keyboard/accessibility; prefill defaults.
- Accept / decline / cancel.

## One active plan

The session runs **exactly one evident plan at a time**. The panel shows the current plan
(collapsed by default; `plan_update` types: items / markdown / file). This is one session's current
plan. If the user starts a new plan, the current goes to draft (kept), then the new plan is active.

## "Transparent, settings-driven" defaults (design 3-state)

Every disclosure uses the same three-state pattern, default-research-why.

| Surface | States | Default |
| --- | --- | --- |
| thinking | collapsed summary / full / hidden | summary |
| tool call rows | expanded / collapsed / hidden | expanded (visible tool) |
| user cards | collapsed / expanded | collapsed |
| token/cost | on / off | **off** |
| research pane | closed / open (button) | closed |

Some defaults are deliberately conservative (the collapsed-state is the "caller can raise" story)
but the entire graded amount is always available.

## Design tokens / interaction conventions (from the desktop handoff)

Follow the shared tokens exactly — the accent, the data ramp, surface hierarchy, hover/pressed/focus,
the 11px floor, Inter 400/500 (no 600+), JetBrains Mono for IDs/paths/models/numerics, Phosphor icons
regular+fill only. Key guideline from desktop to honor:

- **`--ac` (needs-you / act)** for approval, gates, selection, focus — approvals must *look* like
  "act now".
- **`--ac2` (machine working)** for running/pulse/live.
- The accent is never a magnitude fill; the `--data` ramp owns bars/charts.
- Selection = inset ring over `--sf2`.
- Hover lifts a surface; pressed one more; focus is the accent outline.
- Distinction-but-coherent: the agent-panel object families sit on `--sf` cards on `--bg`, with
  hairlines; thinking is a recessed / distinct treatment but within the same palette.

## What's NOT in scope (to avoid confusion)

- The editor pane behavior itself — we only show how opening a file from the panel spawns the pane.
- The full harness/marketplace management; just the panel's harness selector in interplay.
- Shell/Terminal rendering; this surface is ACP events, no terminal emulator in the panel.
- The workflows / agent-fleet surface.

## Review the desktop + v1 handoffs first

Both — prose and tokens — are the source of visual truth. The agent panel must read as a Locus
surface (same family as Plan/Develop), not a clone of any specific external product, though you may
absorb the *interaction* conventions of Zed's/Claude Code's panels noted in the spec.

## Research that shaped these requirements

These are the inputs (protocol + academic + industry) each decision rests on. Not copy — they exist so
you design from the same evidence, not just my conclusions.

### Protocol surface — what the panel renders

- **ACP desktop `session/update` variants** — the panel rows as events: `agent_message` (output),
  `agent_thought` / `agent_thought_chunk` (thinking), `user_message` / `user_message_chunk` (user),
  `tool_call_update` / `tool_call_content_chunk` (tool), `plan_update` (plan),
  `state_update` (running / idle / requires_action), `usage_update` (tokens/cost),
  `available_commands_update` (slash), `config_option_update`, `session_info_update` (title),
  `terminal_*` (display-only; no live terminal in the panel).
  The five object families in the stream **map 1:1** to these event classes — that is why they must
  stay visually distinct: they are different wire types.
- **ACP desktop prompt lifecycle** — turn output is `state_update` + `message`; the panel is an event
  renderer, not a terminal scrollback.

### Permission mode

- **Zed `agent.always_allow_tool_actions`** — a per-run boolean; false = require approval for edits
  and tool calls. This is the A/B bypass model.
- **Claude Code `--permission-mode`** (default / acceptEdits / bypass) — session-scoped modes, the
  origin of the two-permission posture.
- **LangGraph interrupts / Microsoft `approval_mode="always_require"`** — the two gate-bearing designs: Microsoft rides the tool definition, LangGraph rides the graph. Locus rides the allowlist +
  run mode.
- **Zed issue #47695** — users want a **diff preview before approving an edit** (today they approve
  blind and review after). This is why the diff-gate renders inline.

### Thinking / transparency

- **Zed `agent.thinking_display: "default_collapsed"`** + Zed issue #52536 — collapsed "Thought
  process" is the accepted default; expanded-by-default is a reported regression (users scroll
  hundreds of tokens).
- **Claude Code `showThinkingSummaries` / verbose** — summarized thinking, user-settable.
- **2503.14521 (Transparent CoT policy)** — raw chain-of-thought is frequently *inherently
  misleading*; exposure can over-confidence or confuse. Supports summarized-default; the
  user can raise.
- **1811.02164 (Progressive Disclosure)** — incremental transparency can *distract* and undermine
  user heuristics. Base of the collapsed-by-default decision; the user can always raise.

### Progressive disclosure / settings

- **Dustin Kirk, Agentic UX Patterns — Progressive Disclosure Modes** — layered levels (Simple /
  Supervisor), reveal-as-needed. The 3-state (collapsed / expand / drill) pattern.
- **AgentGUI (2607.26300)** — 4 observation tabs at varying detail; user study: **38% faster**
  trajectory comprehension vs flat (p=0.023). Argument for the layered/detail variant approach.

### Research pane / session research

- Not a well-trodden separate-pane pattern in the field (research usually lives in the chat or
  memory) — this is a **differentiating** Locus choice. The session-scoped feed + button-open model.

### Claim / orchestration

- **Claude Code Tasks** (`owner`, `blockedBy`, auto-unblock) — the ownership/claim model. Locus
  already auto-unblocks via graph-generated `blocked_by`; the delta is the orchestrator setting
  owner at dispatch.
- **Cochain (2505.10936)** — multi-agent collaboration "dilutes the primary problem"; assignment is
  cheaper than free-for-all claiming.
- **2508.14635 (Coordination)** — redundant work as a key avoidable cost — what claim prevents.

### Paper library (read fully, for tone)

- **2607.26300 — AgentGUI:** observing + steering long-running agents. Sandbox-per-desk, manager-audit
  loop at ~0.2% of run cost. The closest academic analog to this panel.
- **2601.06223 — Three-Pillar Model:** transparency + accountability as foundations.
- **2606.20630 — Design Principles for Human-Agent Interaction:** 14 principles (negotiate shared
  control, intent transparency, shared repair, tailoring recovery to failure).

### Assessment of evidence

Confidence is **high** on the permission gate, thinking-collapsed, and progressive-disclosure
points — they are backed by shipped reference panels (Zed, Claude, LangGraph) AND the academic work.
Confidence is **medium** on the per-session research pane being a differentiator — the evidence is
that the field mostly does NOT surface research separately, which is an argument for, and a risk.

## Deliverable

Produce a **high-fidelity HTML mockup** (standalone, capable of open) of the agent panel as described:
toolbar, message stream with the **five distinct card families**, the permission-mode diff approval,
the bypass path, the research pane, the plan list, and the context view. Dense but legible; ~ one agent
panel at the width the desktop mockup gives it.

## Open questions for you (not blocking)

- Exact panel layout variants preset names (e.g. density presets) — propose, don't solve.
- Permission-mode menu copy: is "gated" the right word vs "review" in the dispatch toggle.
- Research CLI exact set — nominally out of your scope but worth a footnote.
