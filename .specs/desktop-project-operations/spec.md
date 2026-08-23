# desktop-project-operations

**Milestone** M0.6 · **Depends on** `desktop-application-shell` · **Blocks** M1/M3/M5 project runtime work.

## Purpose

Deliver the desktop project, planning, Automate, and Dispatch contracts rather than treating their screens
as mockups. It owns project configuration, editable plan decomposition, card creation, autorun,
schedules, queue visibility, parallelism policy, and safe global stopping.

## Contract

Projects own harness allow-lists/defaults, repos, base context, extension toggles, and CLI role scope.
Planning has nine stages and commits an editable spec/task-to-card mapping only at final approval.
Dispatch owns queue policy, caps, preemption-at-boundary, autorun, schedules, Stop all, bounded
restore, and a per-job permission posture. Bypass is the default and relies on the declared allowlist
plus container boundary; a user may opt a job into gated approval. Automate renders project
cards/agents from that durable state.

## Acceptance

Project policy is persisted and auditable. Decomposition preserves dependencies. Queued, paused,
running, waiting, stopped, restored, and completed states are distinct. Stop all never deletes a
branch, artifact, memory, or handoff.
