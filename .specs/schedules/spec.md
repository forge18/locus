# schedules

**Milestone** M6 · **Depends on** `workflow-engine`, `run-supervisor`

## Purpose

cron → workflow, recorded against verify results. This depends on a property established at M1: **a
scheduled workflow that only fires while a window happens to be open is not a scheduled workflow**, so
`locusd` outliving the window is the precondition rather than a nicety.

## Governed by

- PLAN.md §M6 — schedules, and overlap skipped rather than queued
- PLAN.md §Process topology — `locusd` outlives the window

## Contract

A cron expression fires a workflow. Executions are recorded with their verify result, so the dashboard
shows green or red rather than "finished".

**Overlap is skipped, never queued.** If the previous execution is still running, the firing is
**recorded as skipped and dropped**. A queue means a slow workflow silently builds a backlog that runs
all at once when it finally finishes — which turns one slow night into a thundering herd at breakfast.

Skips are recorded rather than swallowed: a schedule that skips every firing is a schedule that is
misconfigured, and it should be visible as a number.

## Acceptance

1. A cron expression fires its workflow at the right time with the window closed.
2. An execution records its verify result, not merely that it finished.
3. A firing while the previous execution is still running is **recorded as skipped and dropped** — a
   test proves nothing queued.
4. Skipped firings are visible as a count, not silently discarded.
5. Killing and restarting `locusd` does not lose a schedule or double-fire one.
6. A schedule can be paused and resumed without losing its history.

## Open

- Timezone and DST handling for cron expressions. PLAN.md says nothing, and it is the standard place
  scheduled work goes wrong twice a year.
