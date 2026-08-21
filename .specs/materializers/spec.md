# materializers

**Milestone** M1 · **Depends on** `harness-registry` · **Blocks** `run-supervisor`, `ci`

## Purpose

The code half of the harness contract. Declaring *where* an extension goes is not enough — a harness
with no rules directory needs its rules turned into something it does read, and that is a
transformation rather than a path.

This inverts `local-dx`'s hardest problem. Locus builds the whole config tree per run and throws it
away, so materializers **generate whole files**: no markers, no merge, no prune, nothing to reconcile.

## Governed by

- PLAN.md §Materializers — the six strategies and the plugin contract
- PLAN.md §The one surface — the eight extension types; linters and output-styles as exceptions
- PLAN.md §Token discipline #1 — prefix stability, and why determinism is a token decision

## Contract

**Six strategies.** The first four are parameterized data and name no harness:

| Strategy | Does |
| --- | --- |
| `dir` | copy files as they are, optionally renaming (`suffix`, `flat`) |
| `merged-into` | render files into one target as prose, frontmatter optionally stripped |
| `listed-in` | write the files' paths into a key of the harness's config |
| `entries-in` | convert each file into one structured entry in a config file |
| `plugin` | run an executable that **returns** the files to write |
| `core-driven` | Locus fires the extension itself at boundaries it owns (session start/end from the container's lifetime) |

**The plugin contract** — one executable, JSON-RPC 2.0 over stdio, any language:
```
→ materialize { harness, extension, root: "/locus/config", entries: [{name, frontmatter, body}] }
← { files: [{ path, mode, content }] }
```
**Core writes the returned files** after checking every path resolves under `root`. Three things fall
out of returning data rather than writing it: a materializer is a pure function and testable without a
container, a buggy one cannot escape the config tree, and the same JSON is the fixture for an
event-based "did this harness get its rules" test.

**Byte-determinism is the hard requirement, and it is a token decision.** Sorted file order, sorted
lists inside generated files, no timestamps, no run id, no hostname. The same agent with the same tools
must produce a byte-identical tree, because **that tree is the prompt prefix** and an unstable prefix
costs cache on every run that follows. A materializer embedding the current time is not untidy — it is
a per-run cache miss for every agent that harness serves.

**The config tree is frozen for the life of the run.** Nothing may rewrite it mid-run; editing a skill
affects the *next* run.

**One real plugin ships at M1: pi's.** A generated TypeScript extension is the furthest a harness gets
from copying a directory, so it proves the contract at its hardest point.

## Acceptance

1. All six strategies are implemented in `crates/locus-core/src/materialize/` and name no harness.
2. Materializing the same agent twice produces byte-identical trees — `diff -r` is empty.
3. A materializer that emits a timestamp fails a determinism test.
4. Generated file order and in-file list order are both sorted, asserted independently.
5. A plugin returning a path that escapes `root` is rejected and nothing is written.
6. pi's plugin generates a TypeScript extension that pi actually loads.
7. Every downgraded entry's `weaker_than_native` reaches the materialization report the UI displays.
8. The config tree is read-only for the run's lifetime — a mid-run write attempt fails.

## Open

- Nothing outstanding. The five-vs-six strategy discrepancy in PLAN.md §Materializers was a stale
  sentence and has been corrected to six.
