# screens-develop

> **Historical M0.5 contract.** V2 replaces this fixture's shell and geometry; new work follows
> `.specs/design-v2/spec.md`.

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · View `develop`

## Purpose

The hands-on surface, and the one PLAN.md calls the **primary** editor job: reviewing what an agent
changed. The git panel is the other half — it is read-write on *your* checkout while the branch is the
agent's, which is the whole point of the local-remote git model.

At M0.5 this is the full layout and interaction on fixture data. The real CodeMirror instance and real
LSP arrive with `editor` at M2; what is built here is the frame, the git panel, and a diff renderer
faithful enough to specify against.

## Governed by

- PLAN.md §Editor — one editor at two zoom levels; `MergeView` as the primary surface
- PLAN.md §The git model — clone not mount; you decide what lands
- `.specs/design-v2/spec.md` §Shell and screen inventory

## Contract

Three columns: **206px file tree · editor · 252px git panel**.

**Tree.** Branch header (`git-branch` in accent, mono `agent/8f21-notify`, caret), indented rows at
11.5px with 20px/34px indents, selected file on `--sf2` radius 5 with an accent `M` badge. Footer:
"Linked repo · your own checkout at `~/Repos/tapestry`".

**Editor.** 30px tab bar on `--bg-deep`; the active tab is `--bg` with `inset 0 2px 0 var(--ac)` and a
close `x`; right side `collapseUnchanged` + accent `2 chunks`.

Body is a **side-by-side diff**, mono 11.5px/1.65, 34px right-aligned gutter in `--mu2`, a 12px-wide
± column. Removed rows `rgba(212,97,79,.14)`, added `rgba(79,160,127,.16)`. Collapsed regions are
`⋯ N unchanged lines` strips on `rgba(238,242,246,.03)`. Left header `HEAD · main` in `--mu2`, right
`agent/8f21-notify · builder@4` in accent. Keywords `#8fb8d6`, comments `--mu`.

Footer 52px: secondary "Revert chunk", primary "Open PR from this branch", mono `rust-analyzer · 0
errors · 2 hints`, and the note "Reviewing what an agent changed is the primary editor surface".

**Git panel** (`--bg-deep`, left hairline):

- Header `GIT`, mono `2↑` in accent and `0↓` in `--mu2`, `arrows-clockwise`.
- Branch block: `git-branch`, mono branch, "from main · pushed by builder@4 6m ago".
- `STAGED N` with an accent "Unstage all" and `UNSTAGED N` with "Stage all". Rows are a 9px status
  letter (`M` accent, `A` `--ok`, `?` `--mu2`), a mono truncated path, and right-aligned `+N`/`−N`.
  The current file's row sits on `--sf2`.
- `HISTORY · this branch`: commits as 7px dots (newest accent with a `0 0 0 3px rgba(145,132,217,.16)`
  halo, older `--mu2`), subject, mono `sha · author · age`.
- Footer: commit-message field, primary **Commit**, secondary **Push**, and the note "Working tree is
  your own checkout — the agent pushed to the branch, you decide what lands."

**Stage and unstage work per file and per hunk.** That granularity is the reason the panel exists
rather than a status readout.

## Acceptance

1. Three columns render at the documented widths and are resizable.
2. The diff shows added, removed, and collapsed regions with the exact tints given.
3. The left/right diff headers distinguish `HEAD · main` from the agent's branch, with the agent side
   in accent.
4. Staging a single hunk moves only that hunk between the two sections.
5. Status letters carry their documented colors, and the current file's row is highlighted.
6. History renders newest-first with the accent halo on the newest dot only.
7. The panel's footer note about whose working tree this is renders verbatim — it is the model's
   clearest statement and the easiest thing for a user to get wrong.

## Open

- Whether the editor tab bar supports splits at M0.5 or only at M2 with the real editor. The handoff
  draws a single tab strip and does not say.
