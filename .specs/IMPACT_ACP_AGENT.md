# ACP agent-panel handoff — impact

## Target

`docs/design_handoff_acp_agent_panel/` and the new `agent-interface` contract: a single ACP-session
surface with a docked permission/elicitation blocker, plan dock, composer, checkpoint timeline, and
per-session research pane.

## Dependents

- `acp-client` — must expose plan updates, elicitation, session commands, and panel steering.
- `telemetry` — `permission_request` is conditional: an alarm for a bypass run, a human-action request
  for a gated run.
- `run-supervisor` — must persist the run permission posture, queue prompts at turn boundaries, and
  preserve replay/checkpoint state.
- `pane-manager` — owns the Agent Pane renderer, blocker docking, pane navigation, and detached-window
  event subscription.
- `artifacts` — `finding` artifacts become a session-scoped research feed with provenance.
- `memory` — selected session findings promote at session close; this remains a review action.
- `v2-project-operations` — Dispatch owns the per-job bypass toggle and its auditable default.
- `planning-module` — its findings seed a child task session's research feed.

## Contract conflicts to resolve

1. `acp-client` and `telemetry` currently define every `permission_request` as a misconfiguration alarm.
   The handoff introduces a run-pinned `gated` posture, where it is expected and blocks for the user.
2. M1 is marked closed, but the new M1 `agent-interface` feature has no task list and has not been built.
   M1 must reopen without relabeling the 270 completed runtime tasks.
3. `agent-interface` currently cites only its internal mockup brief. The delivered ACP panel handoff is
   the visual contract and must be cited directly.

## Test coverage gaps

- No Agent Pane tests cover a docked diff gate, elicitation, plan dock, composer steering, research
  provenance, or checkpoint restore.
- No test distinguishes an expected gated permission request from a bypass-mode alarm.
- No test proves a selected finding is promoted only by an explicit session-close review.

## Risk: High

This changes shared event semantics, run state, project dispatch policy, and the primary session UI.

## Recommended action

Add the new `agent-interface` feature to M1; extend the dependent feature contracts and task tables;
update `TODO.md` totals and reopen M1 with its existing 270 tasks still complete. No production code
changes belong in this reconciliation.
