# telemetry

**Milestone** M1 · **Depends on** `harness-registry`, `store` · **Blocks** `run-supervisor`, `ci`

## Purpose

One capture path, one event vocabulary, and nothing downstream knowing which harness it came from.
ACP is the only harness interface, so there is one source with one mapping for every ACP harness. This
is also **how Locus itself is tested**: every run normalizes into the event store, so a test is "run
this, assert these events appeared" — identical across all supported harnesses, with no test-only
instrumentation.

## Governed by

- PLAN.md §Canonical event vocabulary and the single ACP source
- PLAN.md §Harness contract — `[telemetry].source` collapses to `acp`
- PLAN.md §Verification — event-based testing
- PLAN.md §Risks — "Risk — harness surfaces rot, in both halves"

## Contract

**Twelve verbs, and only these:**

```
session_start  user  assistant  thinking  tool_call  tool_result  tool_error
permission_request  subagent_start  subagent_stop  aborted  session_end
```

Every event carries `run_id`, `seq`, `ts`, `stream_pos`, and a **`raw` JSONB of the source record it
was built from**.

**Two orderings, because two questions.** `seq` is total *within a run* and answers "what order did
this transcript happen in". `stream_pos` is total *within a project*, across every run, and answers
"what has happened since I last looked". A per-run counter cannot answer the second without scanning
every run, which is why both exist rather than one.

**Token usage is an attribute, not a verb.** `assistant` and `session_end` carry
`usage {input, output, cache_read, cache_write}` exactly as the harness reports it. **Locus never counts
tokens itself.** Where a harness reports nothing, `usage` is null and spend reads *unknown* rather than
zero — a zero would be a claim the system cannot support.

**`permission_request` is posture-aware.** On a bypass run, every harness launches with its own gate
off, so one firing is a misconfiguration alarm. On a gated run, it is the expected human-action request
that blocks the run until the panel resolves it. The recorded run posture, not the event name alone,
determines which meaning applies.

**One source**, and `[telemetry].source` is `acp` for every supported harness:

| Source | How it arrives | Mapped by |
| --- | --- | --- |
| `acp` | `session/update` notifications on the stream the ACP client holds | **one mapping for every ACP harness**, not one per harness |

**The four-source table is retired.** `hooks`, `stream-json`, and `session-log` no longer feed the
agent surface; there is one path. A harness with no native ACP mode is bridged by a Locus-side mapping,
not dropped to a terminal capture — there is no terminal to capture.

**Teeing stdout has no counterpart here.** `stream-json` teeing mirrored structured bytes to a terminal;
that mirror is gone with the terminal.

**The transcript is a log, and it is deliberately not projected.** `agents.events` is append-only like
`log.entries` and shares its per-project ordering, but it carries **no synchronous projection**: its
aggregates — runs-by-hour, cost per session, verify rates — are computed by query against stored
events. Telemetry is the hot path, written on every tool call of every run, and putting fold work
inside that transaction would tax the highest-volume writer in the system to serve dashboards that
tolerate a query. See `event-store` for the domain log that *is* projected, and why the two
vocabularies stay apart.

**Three rules keep the one path honest:**

- **Ordering is Locus's.** `seq` and `stream_pos` are both assigned on arrival at the core, so a
  source with no ordering guarantee still yields a totally ordered stream. **Neither comes from a
  Postgres sequence.** The core is the sole writer and assigns both from its own monotonic counter
  under one lock, because a database sequence is assigned at insert and made visible at commit — so
  two concurrent runs can commit out of that order and a reader polling `> watermark` silently skips
  the event that committed late. Single-writer assignment is what makes the cursor safe to poll.
- **A missing verb is recorded as missing, never synthesized.** Each harness file declares the verb
  set its source can emit, so a test knows what to expect per harness — otherwise every assertion would
  have to be written to the weakest path.
- **`raw` is kept on every event.** Harness formats change between releases; replay against a fixed
  parser is the repair, and it is why capture is separated from normalization at all.

## Acceptance

1. The `acp` adapter produces events in the shared shape — a test drives an ACP run and asserts the
   vocabulary is the same one any harness's interface would yield.
2. A thirteenth verb is rejected at the type level, not at runtime.
3. `seq` is assigned at the core and is total per run, even from a source with no ordering guarantee.
4. `stream_pos` is assigned at the core, is total per project across runs, and a consumer polling
   `stream_pos > watermark` never misses an event — proven by a test that interleaves two concurrent
   runs and asserts the union of two polls equals every event written.
5. `stream_pos` is treated as strictly-increasing, never as gap-free: a test asserts no consumer
   computes `watermark + 1` or otherwise assumes the next value exists.
6. A harness that cannot emit `thinking` produces **no** `thinking` events — not empty ones.
7. `usage` is null, never zero, where a harness reports nothing.
8. `raw` is present on every event, and a normalization bug is repaired by replay without re-running
   the agent — proven by a test that replays with a fixed parser.
9. A bypass-run `permission_request` raises an alarm; a gated-run request is preserved for a resolvable
   human-action gate rather than being counted quietly.
10. Every supported harness fronts `acp`; a harness with no native ACP mode is bridged by a Locus-side
    mapping that is registered and asserted per harness. No telemetry path renders a terminal.
11. The telemetry append path writes one table and runs no projector — asserted, not assumed.

## Open

- `dx-telemetry` in `local-dx` has absorbed four harness dialects already and PLAN.md names its
  normalization pass as the reference. Whether to port its per-harness tables or rewrite them is an
  implementation call for task 4.
