# agent-session-controls

**Milestone** M1.5 · **Depends on** `acp-client`, `run-supervisor`, `telemetry`

## Purpose

Add the ACP session-control behavior introduced by the Agent Panel handoff without rewriting completed
M1 runtime history: plan projection, elicitation, steering, direct subagents, posture-aware permission
requests, checkpoints, and replay.

## Contract

A dispatch-selected run posture is immutable for that run. `bypass` is the default; an unexpected
permission request is an alarm. `gated` turns the same request into a replayable human-action gate.
Plan updates, elicitation, and session commands are ACP projections consumed by the panel without
widening the canonical event vocabulary. Checkpoint restore and undo preserve the transcript.

## Acceptance

1. Gated and bypass permission requests are distinguishable after replay.
2. A queued steer reaches the next turn boundary; Stop cancels only the active turn.
3. A checkpoint restore or undo never removes transcript events.
4. A panel-created subagent uses the existing bounded invocation limits.
