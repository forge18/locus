# project-search

**Milestone** M2 · **Depends on** `editor`, `store`

## Purpose

Search across a project's repos — plural, because a Locus project holds one or more and the board,
wiki and memory already span all of them. A search that only reaches one repo would be the one surface
that pretends the project is a single checkout.

## Governed by

- PLAN.md §Adding a repo — board, wiki and memory are project-wide
- PLAN.md §Navigation — project is a scope filter, defaulting to all
- PLAN.md §M2 — search across the project

## Contract

Content search across every repo the project holds, plus symbol search where `codanna` has indexed.

**Results carry which repo they came from.** PLAN.md's stated cost of one control plane is that every
cross-project row shows which project it belongs to; the same applies one level down — a result from
`loom-db` and one from `weaver` are not interchangeable.

**Search reads the tree the human is looking at**, not an agent's run clone. An agent searching its own
tree is `rg` in its container, which is a different thing with a different answer.

This is deliberately not the command palette. The palette resolves locators across code, wiki, tasks
and run history; this searches file contents. `command-palette` at M6 is the other one.

## Acceptance

1. A query returns hits from every repo in the project, each labeled with its repo.
2. Results respect the project scope filter.
3. Searching a project with four repos returns results from all four in one list, ranked together.
4. Symbol search returns structural results where indexed, and degrades to content search where not.
5. Opening a result opens that file in the editor at the matching line.
6. Search does not reach into any agent's run clone.

## Open

- Whether `codanna` indexes on a schedule, on demand, or on git change. PLAN.md has it queried live for
  code structure but does not say what triggers an index.
