# repo-manager

**Milestone** M3 · **Depends on** `sandbox`, `store` · **Blocks** `github`

## Purpose

The bare local remote, per-run clones, and merge-back. PLAN.md calls this better than the
shared-worktree approach the rest of the field uses, for three reasons that are worth keeping in front
of whoever implements it — because each one is a temptation to do it the easier way.

## Governed by

- PLAN.md §The git model — a local remote, not shared worktrees
- PLAN.md §Adding a repo — linked and managed modes
- PLAN.md §M3 — merge-back, and conflicts as inbox items

## Contract

```
host: /var/lib/locus/repos/<project>.git        the bare local remote
   │  clone                          push branch
   ▼                                      ▲
container: /workspace  (container-local, not a mount)
   │
   └── you:  git fetch locus && git checkout agent/<run-id>
```

**Two modes for adding a repo:**

| Mode | Where the code lives | You work through |
| --- | --- | --- |
| Linked | `~/Repos/foo` stays yours; Locus syncs with it | your own checkout, and Locus |
| Managed | cloned from GitHub, lives only inside Locus | Locus only |

**Board, wiki and memory are project-wide** and span every repo. That is what makes four repos that
are one system share one memory instead of four that never learn from each other.

**Locus never works in `main`/`master`.** Every agent run branches. The bare remote holds `main` and
nothing Locus does writes to it. **This is an invariant, not a default, and the merge-back path
enforces it.**

**Why this beats shared worktrees, kept here on purpose:**
- **Locus stays out of your editor, your merge tool, and your shell.** Reviewing an agent's work is
  ordinary git, not a bespoke UI you have to trust.
- **Isolation is real.** A bind-mounted worktree can always be escaped by a path bug; a filesystem that
  was never mounted cannot.
- **Nothing to clean up.** A finished container takes its clone with it.

**Two honest consequences.** Two agents *can* both edit the same file — each on its own branch, merging
when done, which is the conflict every team already has. And clones cost disk and time, mitigated by
`git clone --reference` against a shared object store.

**Merge-back:** each agent merges its own branch when done; **a conflict it cannot resolve becomes an
inbox item**, not a silent failure and not a problem left on a branch nobody looks at.

**Involved repos are read-only.** PLAN.md's planning module clones them to `/context/<repo>` beside
`/workspace`, indexes them, and never pushes — the run's branch exists only on the target repo's remote.
Write scope and read scope are different things.

## Acceptance

1. Adding a linked repo leaves the user's checkout untouched and syncs with it.
2. Adding a managed repo clones from GitHub into Locus's own storage.
3. A run clones from the bare remote with `--reference`; N agents on one repo do not mean N copies of
   its history — asserted by measuring disk.
4. An agent pushes a branch back and the user can `git fetch locus && git checkout agent/<run-id>`.
5. **No path writes to `main`** — a test attempts it directly and is refused.
6. Merge-back merges cleanly when it can.
7. An unresolvable conflict becomes an inbox item carrying both sides.
8. Involved repos land at `/context/<repo>`, are never pushed, and a push attempt fails.
9. Three agents work the same repo concurrently from their own clones without interfering.

## Open

- What "syncs with it" means for a linked repo in detail — fetch on demand, on a timer, or on a
  filesystem watch. PLAN.md says Locus syncs with your checkout but not when.
