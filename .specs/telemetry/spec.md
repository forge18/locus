# telemetry

**Milestone** M1 · **Depends on** `harness-registry`, `store` · **Blocks** `run-supervisor`, `ci`

## Purpose

Four capture paths, one event vocabulary, and nothing downstream knowing which path a run arrived
through. This is also **how Locus itself is tested**: every run normalizes into the event store, so a
test is "run this, assert these events appeared" — identical across all twelve harnesses, with no
test-only instrumentation.

## Governed by

- PLAN.md §Canonical event vocabulary and the four-source table
- PLAN.md §Harness contract — `[telemetry].source` as a real switch
- PLAN.md §Verification — event-based testing
- PLAN.md §Risks — "Risk — harness surfaces rot, in both halves"

## Contract

**Twelve verbs, and only these:**
```
session_start  user  assistant  thinking  tool_call  tool_result  tool_error
permission_request  subagent_start  subagent_stop  aborted  session_end
```

Every event carries `run_id`, `seq`, `ts`, and a **`raw` JSONB of the source record it was built from**.

**Token usage is an attribute, not a verb.** `assistant` and `session_end` carry
`usage {input, output, cache_read, cache_write}` exactly as the harness reports it. **Locus never counts
tokens itself.** Where a harness reports nothing, `usage` is null and spend reads *unknown* rather than
zero — a zero would be a claim the system cannot support.

**`permission_request` is a misconfiguration alarm.** Every harness launches with its own gate off, so
one firing means a gate was left on and the run is about to hang with nobody to answer it.

**Four sources**, selected by `[telemetry].source`:

| Source | How it arrives | Mapped by |
| --- | --- | --- |
| `hooks` | `locus-hook` per hook event, JSON on stdin, appended to the run's buffer | hook name → verb, one table per harness. The richest path |
| `acp` | `session/update` notifications on the stream the ACP client holds | **one mapping for every ACP harness**, not one per harness |
| `stream-json` | the harness's newline-delimited JSON on stdout | the core **tees** stdout — structured stream to the normalizer, same bytes to the terminal |
| `session-log` | a file the harness writes; tailed live, re-read at exit | a per-harness parser. Weakest: ordering is file position, `thinking` usually absent, `usage` often only in the final record |

**Teeing stdout is not terminal scraping.** `stream-json` is the harness's declared machine format,
versioned as an interface. A TUI's paint output has no contract and rots every release.

**Three rules keep the four paths interchangeable:**
- **Ordering is Locus's.** `seq` is assigned on arrival at the core, so a source with no ordering
  guarantee still yields a totally ordered stream.
- **A missing verb is recorded as missing, never synthesized.** Each harness file declares the verb set
  its source can emit, so a test knows what to expect per harness — otherwise every assertion would
  have to be written to the weakest path.
- **`raw` is kept on every event.** Harness formats change between releases; replay against a fixed
  parser is the repair, and it is why capture is separated from normalization at all.

## Acceptance

1. All four adapters produce events indistinguishable downstream — a test asserts the same run shape
   from two different sources.
2. A thirteenth verb is rejected at the type level, not at runtime.
3. `seq` is assigned at the core and is total, even from a source with no ordering guarantee.
4. A harness that cannot emit `thinking` produces **no** `thinking` events — not empty ones.
5. `usage` is null, never zero, where a harness reports nothing.
6. `raw` is present on every event, and a normalization bug is repaired by replay without re-running
   the agent — proven by a test that replays with a fixed parser.
7. `permission_request` firing raises an alarm rather than being counted quietly.
8. `stream-json` teeing delivers the same bytes to the terminal and the normalizer.

## Open

- `dx-telemetry` in `local-dx` has absorbed four harness dialects already and PLAN.md names its
  normalization pass as the reference. Whether to port its per-harness tables or rewrite them is an
  implementation call for task 4.
