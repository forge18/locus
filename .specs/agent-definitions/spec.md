# agent-definitions

**Milestone** M1 · **Depends on** `store`, `materializers` · **Blocks** `run-supervisor`

## Purpose

An agent is a Markdown file with frontmatter. PLAN.md calls this **the single biggest simplification in
the design**: every harness already reads `name` + `description` frontmatter over prose, so
materializing an agent is close to an identity transform plus a rename rather than a compiler.

Because agents are not a graph, they land at M1 rather than waiting for the canvas — which is why the
product is usable four milestones earlier than a graph-first design would allow.

## Governed by

- PLAN.md §Agents are Markdown — the frontmatter schema
- PLAN.md §Permissions are declared, never prompted — `tools` as the privilege set
- PLAN.md §Lego blocks — immutable once a run references it

## Contract

```yaml
name: reviewer
description: Read-only critic; runs on task completion
harness: any            # or a specific one, when it matters
model_tier: high        # low | medium | high | xhigh — resolved in Settings; falls back UP
task_class: research    # code | plan | research — sets retrieval depth; defaults to code
tools: [rg, gh, cargo]  # allowlist, resolved against the marketplace index
skills: [audit-code]
rules: [no-secrets]
memory:
  scope: project        # agent | project — never cross-project
  write: propose        # none | propose | direct
```

**`tools` is enforced, not advisory.** It is the container's install set *and* the allowlist. A tool
absent from the list is absent from the image, so the agent cannot reach for it.

**Immutable once referenced.** An edit creates a new version; a run pins the version it used. Editing
`builder` while `builder@4` is running does not change what that run is doing.

**Composition moves to workflows.** An agent does not nest others structurally. It may call `locus
agent invoke` at runtime if that is in its tool list — bounded in the core at **depth 3 and fan-out 4**,
which a workflow may lower and never raise. Depth 3 with fan-out 4 is at most 21 containers, which one
machine survives; depth 4 is 85, which it does not.

**Definitions live in Postgres**, with import and export as `.md` so they can be reviewed in a PR or
copied between machines.

## Acceptance

1. Frontmatter parses and validates; an unknown key is a warning, an invalid enum is an error.
2. `model_tier` outside the four values is rejected; `task_class` defaults to `code` when absent.
3. Saving over an existing name creates a **new version**; the previous one remains readable.
4. A run records the exact version it used, and editing the definition mid-run changes nothing.
5. `tools` naming an entry absent from the marketplace index is rejected at save with the name given.
6. `memory.scope` cannot be set to anything cross-project.
7. Export produces a `.md` that re-imports to an identical definition.
8. Materializing a definition into all twelve harnesses produces the expected file in each layout.
9. Nesting deeper than 3 or wider than 4 is refused by the core, and a workflow cannot raise either.

## Open

- Whether `harness: any` should resolve at run start or be pinned at save. PLAN.md does not say, and
  the difference only matters once one project runs more than one harness for the same agent.
