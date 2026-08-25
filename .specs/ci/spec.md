# ci

**Milestone** M1 · **Depends on** `materializers`, `harness-registry`, `telemetry`

## Purpose

Continuous integration for Locus itself, and one check that is not ordinary CI hygiene: the
**materialization smoke test**. PLAN.md names it "the only thing standing between a harness release and
a silent non-load", which is the failure mode this whole architecture is most exposed to.

Cheap now, and expensive to add after the first silent non-load has been debugged by hand.

## Governed by

- PLAN.md §M1 — CI for Locus itself
- PLAN.md §Risks — "Risk — harness surfaces rot, in both halves"
- PLAN.md §Risks — "Risk — prefix stability decays by accident"
- PLAN.md §Verification — event-based testing

## Contract

**On every push:** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `pnpm build`,
`locus harness lint`, the materialization determinism check, and the per-harness smoke test.

**The smoke test, per harness.** Start a run with a **canary skill and a canary rule**, and assert the
agent can see both. That converts a silent non-load into a failing test, and it is the only reason the
`emits` and `via` declarations are worth writing down at all.

It runs on registration as well as in CI — a harness is not registered until it has passed one.

**The determinism check.** Materialize the same agent twice and assert `diff -r` is empty. PLAN.md's
prefix-stability risk is that *nothing fails when it breaks* — the runs just get more expensive, and
the cause is whatever injection point was added last. The defence is that the determinism check is in
CI and cache rate is on the dashboard, so a regression shows up as a number rather than as a slow drift
nobody attributes.

**Tests assert on the event stream.** Unit tests cover the pure parts; everything above them is "run
this, assert these events appeared", which works identically across the first-party Pi harness and
trusted user harness plugins and needs no test-only instrumentation.

## Acceptance

1. A push runs all six checks and fails on any one.
2. `clippy` runs with `-D warnings` — a warning is a failure, not a note.
3. The smoke test runs **per registered harness**, and a harness that cannot see its canary skill fails
   by name.
4. Deliberately breaking one harness's `via` strategy fails only that harness's smoke test.
5. The determinism check fails when a materializer is made to emit a timestamp.
6. Registering a new harness runs its smoke test before the registration is accepted.
7. CI runs without Docker where possible, and clearly marks the tests that require it rather than
   silently skipping them.

## Open

- Whether CI runs the container-dependent tests on every push or on a schedule. Each harness-plugin smoke
test starts a container, so the tradeoff is real — but skipping them silently is the failure this feature
exists to prevent, so the split has to be explicit either way.
