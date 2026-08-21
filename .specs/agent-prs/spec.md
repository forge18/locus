# agent-prs

**Milestone** M7 · **Depends on** `github`, `artifacts`, `locus-browse`

## Purpose

Agent-authored PRs as a **first-class flow** rather than "the agent ran `gh pr create`". This reuses the
artifact comment machinery — **a PR review comment and an artifact comment are the same thing arriving
from two places** — so it is one implementation, not two.

## Governed by

- PLAN.md §M7 — the five parts of the flow
- PLAN.md §Artifacts — comments steer, and the same mechanism serves both

## Contract

**Open.** The agent's branch becomes a PR with a generated description written from the session's goal,
the tasks it closed, and the evidence it collected. **Screenshots from `locus browse` attach here**, so
a UI change is reviewable without checking anything out.

**Slice.** A large change is split into several reviewable PRs rather than one nobody reads. **PR size
is the strongest predictor of whether a review actually happens.**

**Self-review first.** The agent reviews its own diff and fixes what it finds before asking you. **You
see the second draft, not the first.**

**Respond to review comments.** A comment on the PR routes back into the session that authored it; the
agent pushes updates and replies. **This is the half most tools miss, and it is where the human time
actually goes.**

**Propose merge resolutions.** A conflict comes back as a proposed resolution to accept or reject, not
as a problem handed to you.

## Acceptance

1. A PR description is generated from the session's goal, closed tasks and collected evidence — not a
   diff summary.
2. Screenshots from `locus browse` attach to the PR automatically.
3. A large change is sliced into several PRs, each independently reviewable.
4. Self-review runs before the PR is offered to a human, and its findings are visible.
5. A GitHub review comment reaches the authoring session and produces a follow-up commit.
6. The comment path is the **same code** as artifact comments — asserted by shared implementation, not
   by similar behavior.
7. A merge conflict comes back as a proposed resolution with both sides.
8. A comment arriving after the session's last run exited is delivered by starting the next one.

## Open

- What "large" means for slicing. PLAN.md gives the reason but no threshold, and the wrong one produces
  either one unreviewable PR or five trivial ones.
