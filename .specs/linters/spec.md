# linters

**Milestone** M1 · **Depends on** `materializers`, `agent-cli` · **Blocks** `workflow-engine`

## Purpose

`locus lint` — **the one extension type no harness reads.** The other seven are consumed by the harness;
linters exist so that `locus lint` can find them, which is why **every harness supports linters
trivially and identically** and why the registry has to say that rather than leaving the entry out.

A linter is a check plus **the rule saying why**, per directory. The second half is the point: a check
that fails without saying what it is protecting teaches nothing and gets suppressed. Locus materializes
linters for the first-party Pi harness and for trusted user harness plugins; the CLI contract is shared.

## Governed by

- PLAN.md §The one surface — linters are tool-facing, not harness-facing
- PLAN.md §Agent CLI — `locus lint [--changed|--only NAME]`
- PLAN.md §The Workflow Canvas — `Verify` as the runnable success criterion

## Contract

```
locus lint [--changed] [--only NAME]
```

**Two call sites, and only two:**

1. **The agent, before it commits.**
2. **A workflow's `Verify` node.**

**Never from a hook.** A hook fires on every tool call, so running linters there would tax the whole
run — the same rule that keeps the memory injection path under 100ms and forbids a model call inside a
hook.

**Structure.** One directory per linter: `<name>.sh` performs the check, `<name>.md` says why the rule
exists. Both materialize into `/locus/config/linters/` for every harness through the ordinary `dir`
strategy — this is the one extension where every harness's entry is identical, because nothing consumes
it but Locus.

**`--changed` scopes to the run's diff**, which is what makes it affordable to call before every commit
rather than once at the end.

**Exit code is the result; output is the evidence.** A `Verify` node gates on the exit code, and the
stdout is what lands as the evidence a board transition to Done requires.

**The rule file is returned on failure, not just the check's message.** A linter that fires and prints
only "line 42: bad" costs a lookup; one that prints why the rule exists resolves the question in place.

## Acceptance

1. `locus lint` discovers and runs every linter in `/locus/config/linters/`.
2. `--only NAME` runs exactly one; `--changed` scopes to the run's diff.
3. A failing linter exits non-zero, and a `Verify` node gates on that exit code directly.
4. A failure prints the rule's `.md` alongside the check's message.
5. Pi and a trusted user harness plugin materialize the linters directory identically — a test compares
   the trees and finds them the same.
6. `locus lint` is **never invoked from a hook** — asserted by absence, since this is the kind of thing
   a future convenience adds.
7. A linter with a `.sh` but no `.md` is refused at materialization, naming the missing rule file.
8. Linter output lands as evidence on a board transition.

## Decision

Linters are directory-only in M1. Their materialized `/locus/config/linters/` directory is their
scope; path globs are not accepted. This keeps `--changed` as an input reduction rather than a
second, competing scope model.
