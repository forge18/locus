# agent-interface

**Milestone** M1 · **Depends on** `v2-project-operations`, `pane-manager`, `acp-client`,
`run-supervisor`, `artifacts` · **Blocks** M2 Agent Pane/editor integration

## Purpose

The agent panel surface — how a single agent session renders, and what the user controls and
sees. It consolidates the agent-interface research and the delivered
[`ACP Agent Panel` handoff](../../docs/design_handoff_acp_agent_panel/README.md). This is a
*surfacing* contract over existing machinery, not a second agent protocol: the panel renders the ACP
v2 event stream, permission mode rides the run, thinking rides `agent_thought`, and research rides
`finding` artifacts.

## Governed by

- PLAN.md §What a session is — session / run / turn, one session per container
- PLAN.md §ACP — the only harness interface
- PLAN.md §Permissions — the declared allowlist / container-enforced posture
- PLAN.md §Memory — finding promotion crosses into the long-term store
- `pane-manager` §Contract — Agent Pane renders normalized events; the IPC discipline
- `docs/design_handoff_acp_agent_panel/README.md` — visual and interaction contract; its HTML is a
  reference, never production code

## Governing principle: transparent by default, everything settings-driven

Locus does **not** purposefully hide information. Transparency is the default; the level of detail
shown is *settings-driven*, with research-backed defaults the user can override. A collapsed or
abbreviated default is a **user-overridable control**, never a lock that withholds information.
This applies to every surface — thinking display, tool-call detail, show/collapse/hide — and to
permission mode.

Research basis: raw/trace exposure can distract or mislead (2503.14521, 1811.02164), so defaults
lean conservative — but the user can always *raise* the detail level.

### Visual contract

The ACP panel handoff is authoritative for visual fidelity: Nocturne tokens, a quiet 44px header,
stream-first hierarchy, amber for human decisions, violet for machine activity, and machine strings in
JetBrains Mono. Its HTML and `support.js` are reference material only; Solid components consume the
existing token system and real ACP state.

The panel is one flex-column session surface. The stream is its only flexible region; header, docked
blockers, plan dock, and composer are non-flex and stay visible. Research opens as a toggleable right
column, not a separate monitor screen.

The header carries project, optional task/workflow identity, editable session name, optional cost total,
research, and overflow actions. The composer toolbar carries the agent/model/effort selector, gated-run
Manual/Auto control, and context meter.

## Contract

**The agent panel is the primary session surface.** One panel per session. It renders the ACP event
stream (v2 shapes above) as structured, collapsible, navigable content. Everything below is
settings-driven with research-backed defaults.

### Session controls

| Region | Control | Notes |
| --- | --- | --- |
| header | project, task, workflow, and editable session name | absent task/workflow metadata does not leave a placeholder |
| header | token/cost and Research | cost follows its disclosure setting; Research opens the session pane |
| header | overflow | new session, clear context, checkpoints, and disclosure controls |
| composer toolbar | agent/model/effort selector | one control; available choices follow the selected project and run policy |
| composer toolbar | Manual / Auto | shown only for a gated run; changes subsequent edits, never prior ones |
| composer toolbar | context meter | turns amber above 80%; expands the context view |

The live-status pill is derived: waiting when a gate or elicitation is pending; working when progress
continues with no blocker; otherwise idle. A click locates the current blocker.

**No configurable status line** (Claude Code's `statusLine` is declined). Token/cost and context are
specific controls, not a footer.

### Message stream

The conversation — user input and agent output — as the scroll spine of the panel.

- **User input** — collapsed to a compact card by default; expandable. Editable and resubmittable.
- **Agent output** — one card or a continuous run per turn; supports markdown, code blocks, inline
  file links (clickable → editor pane), tables, images.
- **Diff previews in the approval gate** — when the run is gated and the agent proposes an edit, the
  approval prompt renders the **proposed diff inline**: you see the change before approving, not a
  bare approve/deny. In bypass mode there is no gate, so no diff prompt. Diff review is the gate
  itself — no separate review pane.
- **Docked blockers** — a gate or elicitation docks above the plan and composer, scrims only the stream,
  and may minimize to a one-line pill. The plan collapses while an expanded blocker is present. Gate
  and elicitation stack with bounded internal scrolling; neither can push the composer off-screen.
- **Checkpoint / rollback** — each significant edit gets a "restore" control returning the
  workspace to its pre-edit state. Locus runs on git + per-run clones, so checkpoints are cheap.
- **Tool calls** — shown live with kind, title, status (pending → in progress → completed /
  cancelled), streaming content, collapsible per tool.

### Thinking (internal monologue)

`agent_thought` renders as a **separate, styled, collapsible** block — summarized by default,
expandable to full where the adapter provides it. Never flattened into the message stream, never
expanded by default. Settings: summary / full / hidden (default summary). Raw chain-of-thought is
not a fixed default, per the transparency principle, but the user can raise the level. Hidden or
collapsed thinking is still billed — display is UI-only.

### Elicitation

Adopt the ACP/MCP client surface: restricted JSON Schema (flat object, primitives, enums,
formats), form + URL modes (URL/secrets never enter the client or container). Three response
actions: accept / decline / cancel. Client duties: **validate before sending**, **cache history
and offer suggestions**, keyboard + accessibility per type, pre-populate defaults.

### Plan rendering

The current plan is one plan at a time per session. It renders in the panel from ACP `plan_update`
(`planId`; types: plan / markdown / file). The specific in-panel representation remains open.

### Subagents

Subagents can be created from the panel directly (delegate), not only via a workflow. An ACP
session is not tied to a workflow graph; a user may spawn an agent. *Open: in this spawn's run
semantics should be addressed with run-supervisor (workflow-nullable run).*

### Slash commands

Ship session-level commands: **new-session**, **compact** (context), **clear-context**, plus a
**view context** surface. Discoverable in the command palette / input.

### Research pane — per-session feed

A per-session research feed lives separately from the agent panel — sources + summaries for that
session, collapsible, linkable (claim → source). NOT promoted to the project/wiki (per-project is
too much; the wiki stays the curated project-level home).

- **Opened by a button** — an on-demand toggle, not a permanent split. Research CLIs are
  always available to the research mechanisms — a specific, fixed set (not yet enumerated), stable
  regardless of the bypass toggle.
- **Inherits from planning** — the task's session research feed seeds with the research its
  originating planning session gathered.
- **Promotes to memory at session end** — usable findings can be promoted to long-term memory as a
  review step at session close (not automatic). Distinct from the plan's own memory-of-work.

### Session names

Sessions are named by **task / agent at creation** — the title reflects the task being worked and
the agent. User-editable afterward.

### File links → editor pane

Clicking a rendered file path (outputs, tool calls, diffs) **opens that file in a new editor pane
beside the panel**, not replacing it.

## Settings summary

| Setting | Options | Default | Notes |
| --- | --- | --- | --- |
| thinking display | summary / full / hidden | summary | not hidden by default |
| show tool calls | expanded / collapsed / hidden | expanded | |
| show token/cost | on / off | off | tooltip on demand |
| editable user cards | on / off | on | |
| research pane | on / off (button) | on | this is a session-tool, not default-on |

## Open

- Specific panel layout variants (density presets, Zero's 4-tab stack is a candidate) — see
  *Layout variants* in the design brief.
- Whether permission mode should be called "review" or "gated" per the dispatch toggle.
- The exact always-available research CLI set, checkpoint retention and pruning, workflow-node
  provenance, and agent identity treatment.

## Acceptance

1. A session has exactly one active plan; the panel shows the current one and no two plans run.
2. In permission mode, an edit approval renders the diff inline; in bypass mode, none.
3. Thinking shows summary-collapsed by default, never raw-expanded unless the user raises detail.
4. The research pane is opened by a button, shows that session's findings, and is not project-
   wide.
5. Clicking a file path opens an editor pane beside the agent panel.
6. `token/cost` is a toggle, off by default, panels preserve no other metrics.
7. Elicitation VALIDATES input, shows review, and the form retains prior values after a decline.
8. Slash commands — refers new-session, compact, clear-context, and context — are segmented.
9. A checkpoint returns the workspace to the pre-edit state (git + run clones).
