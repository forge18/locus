# command-palette

**Milestone** M6 · **Depends on** `navigation`, `project-search`, `wiki`, `board`

## Purpose

Discoverability without a second navigation system. The palette and global search **both resolve
locators**, which is the whole design: against one locator scheme they are one resolver with several
callers rather than two navigation stacks that drift.

## Governed by

- PLAN.md §One address space, so there is one resolver
- PLAN.md §M6 — command palette and global search across code, wiki, tasks and run history

## Contract

- **`Cmd-K` resolves a locator.** Type or paste one and go there.
- **`Cmd-P` searches for one** — across code, wiki pages, board tasks and run history, returning
  locators rather than bespoke result types.
- Back and forward per window are a stack of locators.

**Both are callers of the `navigation` resolver, not new navigation.** PLAN.md lists seven entry points
that would otherwise drift apart — the palette, global search, inbox items, board-card links, artifact
comments, notification deep links, and a detached window's identity. Against one locator they are one
resolver with seven callers.

**Search is cross-project by default**, because project is a filter and the cross-project questions are
the useful ones: everything in Reviewing, every idle agent, every guardrail trip today. Every result
shows which project it belongs to — the stated cost of one control plane.

`project-search` at M2 searches file contents in the Develop category; this is the global one. They are
different surfaces, and the palette calls the search rather than reimplementing it.

## Acceptance

1. `Cmd-K` resolves a pasted locator of every kind and navigates there.
2. `Cmd-P` returns results from code, wiki, tasks and run history in one ranked list.
3. Every result is a **locator**, not a bespoke type — asserted on the result shape.
4. Results are cross-project by default and each shows its project.
5. The palette calls the `navigation` resolver; it contains no navigation logic of its own.
6. Back and forward traverse a locator stack.
7. The palette reuses `project-search` for content rather than reimplementing it.
8. An unresolvable locator gives a message naming the bad segment, not a blank screen.

## Open

- Ranking across four very different result kinds. A wiki page, a task, a symbol and a run are not
  comparable by relevance score, and PLAN.md does not say how they interleave.
