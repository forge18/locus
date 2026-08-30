# model-resource-signal

**Milestone** M3 · **Depends on** `context-layer`, `telemetry`, `materializers`,
`run-supervisor`, `guardrails`, `analytics`, `desktop-data-integration` · **Blocks** nothing

## Purpose

Give the agent compact, model-visible resource state before it acts. The signal combines:

1. the previous model call's reported context occupancy, because a high recent value can
   warn that recall is under pressure; and
2. user-defined cost budgets, because the agent must know when continued exploration
   competes with finishing, verification, or handoff.

Raw numbers have no stable behavioral meaning by themselves. One short legend is frozen
near the beginning of context; changing `CTX~` and `BUD$` lines sit at the mutable tail.

This spec amends `context-layer` R5/R6, `telemetry` usage, and the optional run budget in
`guardrails`. It does not claim a calibrated probability of hallucination or a hard
provider billing cap.

## Governed by

- `PLAN.md` §Token discipline — stable prefix, mutable content last, measured accounting.
- `PLAN.md` §Memory — the marketed context limit is not the effective working limit.
- `PLAN.md` §Workflow guardrails — budget accounting, notification, and safe pause.
- `.specs/context-layer/spec.md` R5/R6 — capacity-aware tail placement and recitation.
- `.specs/telemetry/spec.md` — missing usage is unknown, never zero or synthesized.
- `.specs/guardrails/spec.md` — effective values are pinned to a run; pause finishes the
  current turn and leaves the container inspectable.
- `CONTEXT-LAYER-FIT.md` R5 — remaining-capacity policy and mutable-tail placement.

## Current gap

The repository has an optional `token_budget` global default plus schedule override, but
no enforcement path calls the promised 85% pause. It has no cumulative per-task budget,
no project spend budget, and no daily spend budget. The per-project hourly inbox budget
limits autorun count, and the project base-context budget limits authored prompt size;
neither measures model spend.

This feature supersedes the unimplemented token ceiling with a cost-only hierarchy. A
legacy non-null token ceiling is preserved for review and is never reinterpreted as
money.

## Contract

### 1. Context occupancy is not cumulative usage

`ContextOccupancy` is a typed snapshot with:

```text
used_tokens
limit_tokens
reserved_output_tokens
effective_remaining_tokens
warning_at_tokens
compaction_at_tokens
pressure                 N | H | C
measurement_source
observed_model_call
```

`used_tokens` is the most recent per-call `usage.input` value on the normalized ACP event
stream. It describes the **previous reported model call**, not the prompt about to be
sent. It is not session input usage, accumulated spend, cache traffic, or Locus's
estimate. Locus never tokenizes a harness prompt and never sums `usage.input` to derive
occupancy.

`effective_remaining_tokens = max(limit_tokens - used_tokens -
reserved_output_tokens, 0)`. This subtraction is accounting over reported values, not
token counting. The resolved run configuration supplies the model limit, output reserve,
warning point, and compaction point. Core contains no model or harness names and no fixed
threshold table.

All harnesses use the same normalized ACP path. If the latest completed call has no
per-call input usage, or any required value is absent or contradictory, the model-visible
state is unknown. Locus never falls back to an older call and never converts partial data
into normal pressure.

### 2. Cost has one pinned valuation path

`CostObservation` is stored in integer micro-US-dollars:

```text
spend_micros
source                    provider_reported | rate_card_estimate
usage_event               run_id + seq
resolved_model_id
rate_card_version         absent for provider_reported cost
```

Provider-reported cost is authoritative when present. Otherwise Locus multiplies the
harness-reported input/output/cache-read/cache-write usage by a Settings rate card for
the resolved model. A run pins the rate-card version at dispatch, so a price edit cannot
rewrite historical spend or change a live budget.

The Settings rate card stores input, output, cache-read, and cache-write prices per
million tokens. Missing usage or a missing rate makes cost unknown. Locus never substitutes
zero and never estimates from text length.

Each usage event is valued once by `(run_id, seq)`. Replay is idempotent and cannot spend
the same event twice.

### 3. Four optional cost-budget scopes

The user may enable any combination of:

| Scope | Accumulates |
| --- | --- |
| `r` — run | cost in the current run |
| `t` — task | every run and nested-agent descendant attributed to the root board task |
| `p` — project-day | every run in the project during the configured billing day |
| `d` — global-day | every run across all projects during the configured billing day |

A taskless manual run has no `t` scope; it is still counted by `r`, `p`, and `d`. Nested
runs inherit the root task attribution, so delegation cannot escape the task budget.

Settings owns the IANA billing timezone. A day is a local calendar day in that timezone,
including 23/25-hour daylight-saving days; it is never a rolling 24-hour window.

Every scope is optional. Enabling one requires the user to set:

```text
limit_micros
warn_at_percent
act_at_percent
action                    notify | pause | cancel
unknown_cost_action       notify | pause | cancel
```

`0 < warn_at_percent < act_at_percent <= 100`. There are no hidden percentages or default
actions. Settings refuses an enabled budget without a complete policy. Scope-specific
settings override the global budget-policy template and the resolved policy is pinned on
the run.

- `notify` records and surfaces the threshold crossing but continues.
- `pause` lets the current outer turn finish, then stops feeding the loop and leaves the
  container inspectable.
- `cancel` lets the current outer turn finish, records budget exhaustion, and ends the
  run before another turn.

A cost budget is outer-turn-boundary enforcement, not a provider billing cap. Calls made
inside concurrently active turns can overshoot before their usage arrives. Once an acting
threshold is observed, the dispatcher blocks the next affected outer turn or run according
to the resolved action.

### 4. One frozen legend explains both signals

The materializer emits this stable text once near the beginning of assembled context:

```text
CTX~ is previous-call input/limit, not live occupancy; BUD$ is cost spent/limit.
r=run, t=task, p=project-day, d=global-day; D=time until daily reset.
N=normal. H=user warning: verify important claims, persist state, and prioritize completion.
C=configured action due: follow it before broad work. U=unknown. Budgets are ceilings, not targets.
```

The legend is byte-identical across turns and runs with the same materialized inputs. It
never contains counts, thresholds, run ids, timestamps, prices, or harness-specific
wording. It is part of the frozen prefix and causes no repeated prefix-cache invalidation.

### 5. Compact tail lines carry changing state

The mutable tail contains one context line:

```text
CTX~117k/200k; R~74k; N
```

- `~117k/200k` is the previous reported call's rounded-down input/limit in thousands
  of tokens; `~` means last observed, not live.
- `R~74k` is the corresponding approximate remaining capacity after the output reserve.
- `N`, `H`, or `C` is the controller's resolved context-pressure state for that observation.
- Missing latest-call usage renders exactly as `CTX U`.

When at least one cost budget is enabled, one budget line follows:

```text
BUD$ r1.20/2H t4.80/10N p18/25H d31/50N D6h
```

- Amounts are deterministic rounded USD display values; exact micro-dollar values remain
  in the ledger and agent-facing usage query.
- Disabled scopes are omitted.
- `N` is below the user warning, `H` is at/above it, and `C` is at/above the user's acting
  threshold.
- `D6h` is rounded-down time until the next billing-day reset and appears only when a
  daily scope is active.
- An active scope whose cost is unknown renders that scope with `U`; for example `tU`.
  Its configured `unknown_cost_action` applies.

The renderer uses ASCII text, not JSON, XML, prose, or provider-specific special tokens.
The CTX line is at most 32 bytes for supported limits below ten million tokens. The BUD$
line is at most 96 bytes. Exact values remain queryable; prompt values are compact because
single-token and single-micro-dollar precision add churn without changing controller
state. `locus usage --json` returns the exact last-observed input snapshot, exact
spend/limit at every active scope, measurement source, pressure, thresholds, actions,
and next daily reset; unknown numeric values are `null`, never zero.

Changed lines replace their predecessors in the assembled mutable-tail view; the
persisted injection ledger remains append-only. If rounded fields, pressure, and reset
hours are unchanged, no duplicate injection is emitted.

### 6. Pressure has one authority

Context pressure uses the run's resolved ordered thresholds:

```text
0 < warning_at_tokens < compaction_at_tokens <= limit_tokens - reserved_output_tokens
```

- `N` below `warning_at_tokens`;
- `H` at or above `warning_at_tokens` and below `compaction_at_tokens`;
- `C` at or above `compaction_at_tokens`.

Budget pressure uses each scope's user-defined `warn_at_percent` and `act_at_percent`.
The prompt renderer, guardrail controller, telemetry query, and future UI consume the
same resolved states. None reimplements thresholds.

When multiple budgets act together, every configured action is recorded and the strictest
wins for execution: `cancel` > `pause` > `notify`. No scope can loosen another scope.

### 7. Injection and provenance

The supervisor injects the latest shorthand at the next outer-turn boundary through the
same bounded tail path as recitation. The CTX~ and BUD$ lines follow the recitation block.
Injection never calls a model and inherits the hook path's 100ms and exit-0 discipline.

Each emitted line is recorded with its exact last-observed input and budget snapshots in
the existing context-injection/materialization ledger used by `context_attribution`.
This adds no telemetry verb. Replay can answer what shorthand the model saw, which
measurements produced it, and which budget action was active.

The latest completed call's ACP usage becomes visible on the next outer turn. If that
call reports no input usage, CTX is unknown. Cost budgets update when the corresponding
usage event is persisted; the next outer turn sees the new aggregate.

## Non-goals

- No claim that context occupancy maps to a universal hallucination probability.
- No Locus-side prompt tokenization and no occupancy inference from accumulated usage.
- No token-denominated run/task/project/day spending budgets; budgets are cost-only.
- No hard provider billing cap or interruption of an in-flight model request.
- No per-harness capability matrix, adapter branch, or harness-specific verification.
- No fixed warning/action percentages in core; Settings is the authority.
- No Analytics or Agent Pane presentation in this feature. Settings configuration is in
  scope and must use the live command/provider seam from `desktop-data-integration`.

## Acceptance

1. `ContextOccupancy` uses only the latest completed call's normalized ACP `usage.input`,
   never sums usage, never tokenizes a harness prompt, and labels the result last-observed.
2. Missing latest-call usage, model limit, or reserve renders `CTX U`; no older call is
   substituted.
3. Every harness passes through the same normalized ACP path; core has no capability
   matrix, harness-name branch, or per-harness test suite for this feature.
4. Provider cost wins over estimated cost; estimates use a run-pinned model rate card and
   harness-reported usage only.
5. Missing usage or pricing makes cost unknown, never zero; replay cannot double-count a
   usage event.
6. Run, root-task lineage, project calendar day, and global calendar day produce the four
   aggregates without cross-project or cross-day leakage.
7. Nested agents spend the root task budget; taskless runs omit only the task scope.
8. Daily reset follows the Settings IANA timezone across daylight-saving boundaries.
9. Settings refuses an enabled budget without its limit, warning threshold, acting
   threshold, action, and unknown-cost action; no percentage or action is hidden in core.
10. Simultaneous threshold crossings apply the strictest action, and a looser scope never
    weakens a tighter one.
11. Notify, pause, and cancel act at the next outer-turn boundary and record the scope
    that triggered them.
12. A legacy non-null token budget is preserved for review and never converted to USD.
13. The frozen legend is emitted once, is byte-identical across turns, and says CTX~ is
    previous-call data and budgets are ceilings rather than targets.
14. The compact renderers produce the specified CTX~ and BUD$ forms within 32 and 96 ASCII
    bytes; missing context renders `CTX U` and unknown active spend renders scope `U`.
15. Updating either line changes only the mutable tail; the frozen head remains
    byte-identical.
16. Unchanged rounded fields and states emit no duplicate injection; a pressure transition
    emits even when rounded values are unchanged.
17. The injection ledger records rendered shorthand, exact producing snapshots, and active
    actions without adding a thirteenth telemetry verb.
18. `locus usage --json` exposes the exact typed state and preserves unknown numbers as
    `null`.
19. Settings configures rate cards, billing timezone, every budget scope, thresholds,
    actions, and unknown-cost actions through live commands rather than fixtures.

## Verification

```text
verify: cargo test -p locus-core model_resource_signal
```
