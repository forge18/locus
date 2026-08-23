# agent-interface — design notes (research in progress)

> **Status: research, not contract.** These are decisions reached during the agent-interface
> research pass, captured so they are not lost. They are NOT yet the spec. Each will move into its
> owning feature spec (below) when the research round closes and the design is approved. Editing
> this file is not the same as settling a spec.

## Context

We researched ACP (Agent Client Protocol) v1/desktop, the reference client UIs (Zed's Agent Panel,
JetBrains AI Chat, Grok Build), and the academic/industry literature on agent interfaces. The goal
was to pin down how agent panels, permissions, thinking display, and scheduling should behave in
Locus. The findings below are the converged answer to the six open threads; each carries its named
source and its eventual spec home.

## Governing principle: transparent by default, everything settings-driven

**Status: decided.**

Locus does **not** purposefully hide information. Transparency is the default; the level of detail
shown is *settings-driven*, with reasonable research-based defaults the user can override. This
applies to every surface — thinking display, show/collapse/hide, tool-call detail — and to permission
mode.

- A collapsed/abbreviated default is a **user-overridable control**, never a lock that withholds
  information.
- Research-backed defaults exist (collapsed thinking, whole-run bypass) but nothing is hidden from
  the user who asks for it.
- The transparency research (2503.14521, 1811.02164) warns raw exposure can distract or misfire
  — so defaults lean conservative — but the user's ability to *raise* the detail level is never
  removed.
- *Owned by:* `theme-system` (defaults) + every surface that shows agent detail.
- *Restates:* the user's explicit position — "I will not purposely hide information. Transparent,
  settings-driven, reasonable research-based defaults."

## Decisions

### 1. Bypass-permissions toggle lives in Dispatch, per job

**Status: decided.**

When scheduling a job in Dispatch, a per-job boolean toggles the run between the two permission
postures:

| Toggle | Mode | Behavior |
| --- | --- | --- |
| **bypass permissions** (default) | **A** — declared + container-enforced | Allowlist is the boundary; no prompts. Matches "you are never prompted." |
| **off** | **B** — permission-gated | Protected actions block on `locus ask`-style approval; run enters `waiting`. |

- **Default is ON** (bypass), preserving the "never prompted / a run that stops to ask has nobody
  watching it" invariant. Gating is an explicit per-job opt-in, not a default that can hang.
- **Whole-run granularity** — one boolean, not per-tool-class (explicitly "no C"). Mirrors Zed's
  `agent.always_allow_tool_actions`.
- **Slot**: records on the run/session row at schedule time. Reuses the existing `waiting` mechanism
  (`locus ask` already blocks, sets `waiting`, reaches the inbox).
- **Event meaning flip (scoped to a bypass-off job):** `permission_request` reads as "needs you"
  (real approval → inbox), not "a gate got left on" (misconfiguration alarm). This is scoped to the
  job's mode, not a global redefinition.
- *Owned by:* `design-desktop`/`desktop-project-operations` (Dispatch surface).
- *Source:* Zed `agent.always_allow_tool_actions`; Claude Code `--permission-mode`; LangGraph
  interrupts / Microsoft `approval_mode="always_require"`.

### 2. Agent panel themes are layout + look-and-feel variants, not visual themes

**Status: decided (expanded).**

"Theme" for the agent panel means the **look-and-feel + interaction** of the agent surface — pi looks
different from Claude Code from Codex — not just density/arrangement, and not the dark/light skin.
A small set of panel variants capture each product's terminal language and density (arrangement,
detail). The dark/light skin stays under the existing `theme-system`; the look-and-feel is what
panel themes mean.

- **Show / collapsed / hidden** is per-surface, three states, user-settable, in the progressive-
  disclosure structure. Summary → expand a step → expand a step's input/output.
- Layout variants should span *detail levels* (AgentGUI's 4-tab stack) plus product flavor.
- *Owned by:* `pane-manager` (layout variants) + `theme-system` (visual colors only).
- *Source:* Dustin Kirk, Agentic UX Patterns — Progressive Disclosure Modes; AgentGUI observation
  tabs; the pi / Claude Code / Codex product difference themselves.

### 3. Internal monologue renders as summarized, collapsible thinking (Claude Code pattern)

**Status: decided.**

Thinking is the Claude Code pattern, not raw chain-of-thought:

- Render `agent_thought` as a **separate, styled, collapsible** block, default-collapsed, with an
  expand affordance — never flattened into the message stream, never fully-expanded by default.
- **Summarized by default, not withheld.** The industry norm (Anthropic et al.) is summarized
  thinking blocks, and raw chain-of-thought is unreleased — but per the governing principle this is
  a *settings-driven default*, not a wall. The detail level (summary / full / hidden) is user-
  settable, with collapsed-summarized the research-backed default.
- **Billing caveat:** hidden/collapsed thinking is still billed — display is UI-only, does not
  reduce cost.
- **Broad harness support:** almost all harnesses expose summarized thinking through their own
  settings — the summary layer is available, not a rare capability.
- *Owned by:* `acp-client` (thinking chunks), `pane-manager` (rendering).
- *Source:* Zed `agent.thinking_display: "default_collapsed"`, Zed issue #52536 (expanded-by-default
  is a reported regression); Claude Code `showThinkingSummaries` / abbreviated-vs-verbose; ACP
  `agent_thought` content type.

### 4. Claim-based task ownership (orchestrator-scoped)

**Status: decided.**

Not every agent claims freely — the workflow/orchestrator owns assignment. The delta over the
existing board is that **the workflow node sets `assigned_agent` + `session` (owner) before it
dispatches the agent**, and parallel branches skip already-claimed work. Check-before-claim matters
because Locus runs N concurrent agents.

- Locus already exceeds this on the unblock side: `blocked_by` is workflow-graph-generated and
  "Blocked clears automatically, never manually" — the auto-unblock Claude Code popularized is
  already here.
- *Owned by:* `workflow-engine`, `board`.
- *Source:* Claude Code Tasks (`owner`, `blockedBy`, auto-unblock, claim-before-start).

### 5. Elicitation client surface (agreed)

**Status: agreed.**

Adopt the ACP/MCP elicitation client surface: restricted JSON Schema (flat object, primitives,
enums, formats), **form + URL modes** (URL/secrets never enter the client or container — aligns
with the credential broker), three response actions (`accept` / `decline` / `cancel`), and the
client-side duties: validate before sending, cache elicitation history + offer suggestions, keyboard
shortcuts + accessibility per type, pre-populate defaults.

- *Owned by:* `acp-client`.
- *Source:* MCP elicitation spec (draft client); ACP elicitation RFD.

### 6. Per-run environment artifact — withdrawn

**Status: retracted.**

Earlier proposed a generated per-run ENV/catalog artifact listing repos, tools, services. Retracted:
the container only ever contains permitted repos/tools, so **the filesystem is the inventory** —
presence is the permission. Nothing needs declaring. Only write-vs-read scope and service *names*
(reachable but not mounted) aren't conveyed by presence, and that is too thin to justify an artifact.

- *Owned by:* n/a.
- *Source:* design review with the Locus author.

## Panel surfaces (round 3)

**Status: decided.**

Drawn from the full ACP desktop surface + reference panels (Zed, Claude Code). This is what the agent
panel renders/links; it extends the decisions above.

### Diff preview in the approval gate

In permission mode (bypass OFF), the approval prompt **renders the proposed diff inline** — the user
sees the edit before approving, not a bare approve/deny. Only meaningful when permissions mode is
on; in bypass mode there is no gate, so no diff prompt.

- **Diff review is not a separate surface.** It is the same as the gate — the inline diff in the
  approval flow. No separate review pane beyond.
- *Owned by:* `pane-manager` (render) + the permission gate in review mode.
- *Source:* Zed issue #47695 (approve blind, review after); Claude Code IDE "live diff overlay".

### Slash commands — session-level

- Ship slash commands for **new-session**, **compact**, and **clear-context**.
- Also need a way to **view context** — a surfaced context view (what is in the window / memory).
- *Owned by:* `acp-client` (commands) + `pane-manager` (rendering).

### No status line

- Declined. The panel does NOT get a configurable status line (Claude Code's `statusLine`).
- Token/cost and other per-output info remain as already listed; no overall footer status line.

### Checkpoint / rollback

- **Yes.** Every time the model edits, return to the pre-edit state via "Restore Checkpoint" (Zed's
  pattern). Locus's git model + run cloning makes this nearly free.
- *Owned by:* `artifacts` / `repo-manager` (checkpoint) + `pane-manager` (button).
- *Source:* Zed "Restore Checkpoint".

### Plan rendering — needs design

- Plan display within the panel is still open (current plan, given one-plan-at-a-time).
- ACP desktop has `plan_update` (planId, types: items / markdown / file). To be designed separately;
  flagged as open exploration.

### Session replay

- In scope. Session re-attach / replay (ACP desktop's own purpose) aligns with the session/run model.

### Subagents

- **Yes — subagents can still be created.** An ACP interface need not be tied to a workflow. A user
  can spawn a subagent from the panel directly (delegate), independent of the workflow graph.
- *Owned by:* `acp-client` / `run-supervisor`.

### Research pane — per-session feed

**Status: decided (per-session).**

A session research feed lives separately from the agent panel — sources + summaries + the agent's
derivations for THAT session, collapsible, linkable (claim → source). NOT promoted to the project
/wiki (per-project is too much); the wiki remains the curated project-level home.

- This is a *surfacing* decision over existing storage, not new schema: `finding` artifacts are the
  per-session home; wiki `source` pages are the project home. The pane is the session focus over
  `finding`.
- Web research: separate per-session research panes are NOT a strong existing pattern — the field
  mostly keeps it in the chat or in memory. So this is a differentiating choice, not a benchmark.
- *Owned by:* `pane-manager` (view) + `artifacts`/`finding` (backing).
- **Opened by a button** — the research feed is an on-demand toggle, not a permanent split. A
  visible research/evidence button in the agent panel opens the session research feed beside it;
  closing returns to the agent panel alone.

- **Inherits from planning.** A task's session research feed seeds with the research its
  originating planning session gathered — the planner's `finding`s flow into the task's feed, so
  the executor starts from the same evidence base the plan was built on, not from scratch.
- **Promotable to memory at session end.** Findings are session-scoped working research, but
  usable findings can be promoted into long-term memory when the session closes — the bridge
  from per-session working research to durable project memory. Promotion is a review step at
  session close, not automatic. Distinct from the wiki (curated project pages) and from the
  planning-inheritance seed (parent → child session feed).

### Always-available research CLIs

A **specific, fixed set of CLIs is always available to the research mechanisms** — not contingent on
per-run permits. Independent of the bypass toggle: research has a stable tool floor.

The exact set is not yet enumerated (a follow-up); the guarantee is the binding constraint. This
meets the earlier "available CLIs" injection need for research without a per-run catalog artifact —
it is a fixed, declared set owned at research surface, not by the per-run environment.

- When file paths render (outputs, tool calls, diffs), clicking one opens a new editor pane next
  to the agent panel with that file.
- *Owned by:* `pane-manager` (navigation).

- **C: calibrated three-tier human-in-the-loop** — too much. A and B only.
- Multiple concurrent plans in a session — the "one plan at a time" invariant (see
  `planning-module` §Contract / acceptance 14) is retained.

## Academic corroboration

Research round 2 surveyed the academic literature. It corroborates the decisions above and adds
evidence:

- **AgentGUI** (2607.26300): an interface for observing and steering long-running agents. Its 4
  observation tabs at varying detail (activity feed → overview → turn-level token telemetry →
  code-only console) are a concrete instance of the layout-variant idea. User study: **38% faster**
  trajectory comprehension vs. a flat dashboard (p=0.023) — evidence *for* layered observation.
  Steering via mid-turn input, edit-task-reviewed-at-turn-boundary, and **profile-swap mid-task**.
  Sandbox-per-desk == Locus container-per-run. Its manager-audit (decompose, gather evidence,
  judge, report, resume) is the verification/guardrail loop, at **~0.2% of agent tokens** (162-316k
  manager vs tens of millions agent).

- **Three-Pillar Model** (2601.06223): progressive validation akin to staged autonomous driving;
  transparency + accountability foundational. Supports the permission-tier view.

- **Transparent CoT policy** (2503.14521): raw chain-of-thought is frequently *inherently
  misleading* (overthinking, performative multi-lingual reasoning) and full disclosure causes
  overconfidence/misinterpretation. Proposes **tiered-access**: who sees raw vs summarized reasoning
  is a permission tier, not a UI default. This reframes the thinking decision: the level of CoT one
  is shown could follow the bypass/permission tier, not a fixed UI default.

- **Progressive Disclosure** (1811.02164): incremental transparency can be *distracting* and
  undermine the simple heuristics users form — users anticipated it would help, then retracted. Supports
  default-collapsed posture (isn't taste, it's evidence).

- **Cochain** (2505.10936): multi-agent collaboration "consumes substantial tokens and inevitably
  dilutes the primary problem." Argues against free-for-all agent coordination; assignment is
  cheaper.

- **Coordination** (2508.14635): measured **redundant work** as a key coordination metric — exactly
  the duplicate-work the claim/assignment mechanism prevents.

**Case against the open items:** (a) approval granularity remains whole-run (no research argues for
per-class); (b) layout variants should include *detail levels* (AgentGUI's 4-tab stack) *plus* the
product look-and-feel (pi vs Claude Code vs Codex), not just compact vs verbose; (c) thinking
summaries are broadly available — almost all harnesses expose them via their own settings — and CoT
tiered-access resolves into a per-user display setting (raw vs summary), not a permission gate.

## Open (still researching)

- Exact set of panel layout + look-and-feel variants (AgentGUI's 4-tab stack is a candidate shape;
  product flavor per the pi / Claude Code / Codex difference).

## Decision log

| # | Thread | Decision | Status |
| --- | --- | --- | --- |
| 1 | Permission mode | Dispatch per-job bypass toggle (A/B, whole-run) | decided |
| 2 | Panel themes | Layout variants, not visual | decided |
| 3 | Thinking | Summarized, collapsible (Claude Code) | decided |
| 4 | Board/ownership | Orchestrator claim + check-before-claim | decided |
| 5 | Elicitation | ACP/MCP client surface | agreed |
| 6 | Env artifact | Withdrawn (filesystem is the inventory) | retracted |
