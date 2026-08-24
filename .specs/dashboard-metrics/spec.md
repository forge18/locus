# dashboard-metrics

> Superseded by `analytics-revision` for the current Analytics surface; its metric definitions remain valid.

**Milestone** M6 · **Depends on** `telemetry`, `workflow-engine`, `screens-dashboard`, `screens-review`

## Purpose

Every metric here is **already a column**, so this is a set of queries rather than new instrumentation.
That is the point: the design put the numbers in place while building the thing, so the dashboard costs
a `GROUP BY` instead of a subsystem.

## Governed by

- PLAN.md §M6 — the metric list
- PLAN.md §Token discipline — cache rate and the offender ranking
- PLAN.md §When `Verify` fails — the arbiter metrics that separate a bad builder from a bad spec

## Contract

| Metric | Source |
| --- | --- |
| Runs, spend | run rows and `usage` |
| **Cache rate** | `usage.cache_read` against `usage.input` |
| **Tool-payload offender ranking** | `GROUP BY` over `tool_result` rows, per agent, project and harness |
| Verify pass rate | verify results |
| Guardrail trips | trip rows |
| Board throughput | transitions |
| **Spec-gap rate**, **ambiguity-detection rate** | the arbiter's classification column |
| **Average iterations per task**, **review-gate precision** | iterations and gate outcomes |
| **Agent trust** | verify pass rate over the last 20 runs, discounted by guardrail trips, by artifacts a human rejected, and **by tokens spent per passing run** |

**The arbiter metrics separate a bad builder from a bad specification.** A workflow that keeps producing
spec gaps is visibly a planning problem, and without that split every failure looks like the builder's.

**Cache rate is a column, not a project.** Below ~80% on a long session means something in front is
moving, and the run that did it is identifiable. This is the tripwire for PLAN.md's stated risk that
prefix stability decays by accident — **nothing fails when it breaks, the runs just get more expensive**.

**Agent trust is weighted by tokens per passing run**, because a run that passes verify on four times
the tokens is a worse run wearing a green tick.

**Where these render.** Status is the at-a-glance half and **deliberately does not grow a query tool**;
Review is where you dig. Keeping them apart is what stops Status becoming a second Review.

## Acceptance

1. Every metric is a query over existing rows — a test asserts no new write path was added for any of
   them.
2. Cache rate computes from `usage.cache_read` over `usage.input`, and reads *unknown* where usage is
   null rather than 0%.
3. The offender ranking orders tools by total result payload and slices by agent, project and harness.
4. Spec-gap and ambiguity rates come from the arbiter's column.
5. Agent trust discounts by all three factors, and the token-per-passing-run term is present.
6. Status renders the at-a-glance set with no search, filter or facet control.
7. Review renders the full set with facets.
8. A deliberately unstable prefix drops cache rate below 80% and the responsible run is identifiable.

## Open

- The cache-rate alert threshold. PLAN.md says "below ~80% on a long session", but "long" is undefined
  and a short session legitimately has a low cache rate.
