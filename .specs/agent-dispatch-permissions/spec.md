# agent-dispatch-permissions

**Milestone** M1.5 · **Depends on** `desktop-project-operations`, `agent-session-controls`

## Purpose

Expose and persist the Agent Panel handoff's whole-run permission choice in Dispatch. Bypass is the
default; gated approval is an explicit per-job opt-in.

## Acceptance

1. A scheduled job records one permission posture.
2. Gated requests are shown as waiting human actions, not alarms.
3. Dispatch explains the choice before a job starts.
