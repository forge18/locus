# github

**Milestone** M7 · **Depends on** `repo-manager`, `board`

## Purpose

Version control, CI/CD, PRs — **and Issues as an input to the Locus board. Never GitHub Projects**; the
name collision with a Locus project is unfortunate and means nothing.

## Governed by

- PLAN.md §M7 — GitHub, and Issues linked by explicit action
- PLAN.md §Decisions already made — GitHub row
- PLAN.md §The board — `github_issue` on the task

## Contract

`gh` + `gix` in core: branch, PR open/review/merge, CI status. CI status surfaces on the board and the
dashboard.

**GitHub Issues, linked by explicit action in either direction. No background sync, no polling, no
conflict resolution** — the link is established by a person, once:

- **Attach an existing issue** to a Locus task → imports its title, body and labels **at that moment**
- **Create a GitHub issue** from a Locus task → pushes it out and records the link
- Either way the task carries the issue number and URL, and the PR closes it with `Fixes #142`

**Nothing syncs in the background, and that is the point.** Every tracker integration that tries to keep
two systems continuously equal ends up owning a conflict-resolution problem nobody asked for.

**Locus opens the PR.** How that PR reaches `main` — which branch it comes from, who merges it, and
under what checks — follows the project's own convention, not a rule Locus imposes.

## Acceptance

1. `gh` and `gix` operations work from core without a shell wrapper leaking into callers.
2. Attaching an issue imports title, body and labels **once** and never again.
3. Editing the GitHub issue afterwards does **not** change the Locus task — asserted, since this is the
   behavior a future contributor will be tempted to "fix".
4. Creating an issue from a task records the link in both directions.
5. A PR body carries `Fixes #N` when the task has a linked issue.
6. CI status appears on the board card and the dashboard.
7. No polling loop and no background sync job exists — asserted by absence.
8. Locus never merges to `main` itself.

## Open

- Rate limiting and auth for `gh` when several projects are active. PLAN.md routes service credentials
  through the broker, but does not say whether the GitHub token follows the same path.
