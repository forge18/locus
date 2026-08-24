# workshop-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision`, `setup-revision` · **Blocks** M4 workflow canvas and M8 marketplace installer

## Purpose

Workshop is the meta-harness: the one place every extension is authored once and materialized fresh
into whatever a harness reads. `design-revision` fixed the rail vocabulary and the 29-view inventory
but explicitly deferred Workshop's own contract — nine extension types sharing one editor, plus
Harnesses, Providers, CLI, and Workflows as bespoke screens — to this feature, and left one open
question for it to close: the autorouting bands read `minimal` while Plan → Decompose cycles
`low / medium / high / xhigh`. This spec is the full contract for all twelve Workshop views and settles
that vocabulary.

It changes no runtime behavior beyond what its own tasks implement; the screen-by-screen prose lives in
[`docs/UI_MOCKUP_REVIEW.md`](../../docs/UI_MOCKUP_REVIEW.md) § Workshop, and this spec does not restate
it — it extracts the parts that bind: field kinds, gating rules, and the boundary between what the UI
draws and what `locus-core` already enforces.

## Governed by

- `PLAN.md` §The one surface — the eight extension types, and the two that don't behave like the rest
- `PLAN.md` §Materializers — the code half of the contract
- `PLAN.md` §Model routing — mechanism in the file, policy in the UI
- `PLAN.md` §Marketplace — the local manifest index CLI tools resolve against until M8
- `docs/UI_MOCKUP_REVIEW.md` § Workshop — the reviewed contract for all twelve views
- `.specs/design-revision/spec.md` — Decision 3 (the registry decides the harness roster) and the
  `## Open` item this spec resolves

## Contract

### The shared extension editor

One three-pane component serves nine types: **skills, rules, context, commands, hooks, styles,
linters, agents, harnesses**. A type is a fixture switch on the component, not nine components.

- **Left rail** — icon, label, total count, a one-line blurb, **New `<singular>`**, a sort control,
  the item list, and a footer naming the storage unit (`One directory per skill, entry point
  SKILL.md.`).
- **Center** — title and meta, **History** and **Save**. Save is manual on every type in this group —
  the contrast is deliberate: Workflows autosaves, the extension editor does not, because an extension
  edit is reviewed before it reaches a materialized tree and a workflow edit is not. Up to six blocks
  follow: a frontmatter key/value table, an autorouting section (harnesses only), an adapter-config
  table (harnesses only), an optional segmented field, an optional checklist, and the rendered file
  body.
- **Frontmatter table** — field kind (`text`, `select`, `number`, `toggle`, `chips`) is inferred from
  the field **name**, not declared per type: a name ending in `_tokens` or matching a numeric budget
  renders `number`; `tools`, `roles`, and other list-shaped names render `chips`; a name whose value
  set the registry can enumerate (harness, provider) renders `select`; anything boolean-named renders
  `toggle`; everything else renders `text`. One inference table, shared by all nine types.
- **Right — Materialization** — native and downgraded counts derived from the harness registry (never
  a literal, per `design-revision` Decision 3), a per-harness segment bar, the downgrade explanation,
  a byte-determinism note (`Sorted order, no timestamps, no run id. The materialized tree *is* the
  prompt prefix, so an unstable one costs cache on every run that follows it.`), and version history.
  **Harnesses is the one type with no Materialization rail** — a harness record configures the
  mechanism, it is not itself materialized into one.

Per-type contract detail, carried from the review because each is a distinct invariant a test must
hold:

| Type | Contract detail |
| --- | --- |
| skills | Lazy-loaded. A `budget_tokens` field refuses to materialize the skill above that cap. Downgrade inlines the description into base context and loses lazy loading. |
| rules | One glob per rule — a second match is a second rule, not a second glob. A `priority` field orders overlapping matches. The most downgraded type: concatenated into base context, firing on every file. |
| context | Exactly one per project. Native everywhere — it is the fallback every other downgrade lands in. Over budget means something upstream was downgraded and should be fixed there instead. |
| commands | Argument-taking prompt templates. A command with no args should be a skill instead. Downgrade materializes it as a skill, losing argument validation. |
| hooks | One event, a threshold, a timeout, and an on-error choice ending in exit 0 / log / continue. A hook that fails a run has turned an optimisation into an outage. |
| styles | Exactly one active per harness. A roles checklist decides which roles get which style. The largest single downgrade in the system when merged into base context. |
| linters | Native 0 / downgraded 0, by design — a human-and-CLI surface (`locus lint`), not materialized into any harness, so there is nothing to downgrade. A violation choice of `warn` or `fail`; a rule nobody can fail is a preference. |
| agents | Frontmatter plus a tool allowlist. The allowlist **is** the privilege set: changing it rebuilds `locus/agent-<hash>` and invalidates the prefix cache. Editing the prose below rebuilds nothing. Native in every harness. |

### Harnesses

Harnesses has no Materialization rail (above) and no downgrade vocabulary — "All eight extension types
are supported on every harness. What differs is the mechanism, and the mechanism is the adapter's
problem, not yours."

- **Record** — `identifier` (must match the CLI on `PATH`), `adapter` (`HarnessDescriptor.adapter` in
  `crates/locus-core/src/harness/selection.rs`; **no adapter, no selection, anywhere** — enforced today
  by `ProjectHarnessPolicy::select` returning `AdapterUnavailable`), `providers` (chips restricted to
  those configured under Providers — `ProjectHarnessPolicy::configured_providers` gates this the same
  way it gates project selection), `default model`, `default effort`.
- **Adapter config** — a free-form `Key · Value · Type` table: keys the adapter reads that later config
  lands in without a schema change. No `locus-core` struct backs this today; it is a JSONB blob keyed
  by harness, read by the adapter and by nothing else in core.
- **Autorouting** — a per-harness on/off switch (`AutoroutingPolicy.enabled` in
  `crates/locus-core/src/runtime/routing.rs`). Off: every task runs on the record's default model and
  effort. On: the six-band table (`ComplexityBand`: `xtra-low, low, medium, high, xtra-high, max`),
  each row `Complexity · Model · Effort · Approval · When to use it`. **A band with no model set never
  receives work: the task falls to the next band up** — `AutoroutingPolicy::route`'s upward-fallback
  loop, unchanged. **Sizing happens once, when the card reaches the board — not per iteration.**
  Models come from the harness's configured providers; an alias set under Providers is what the band's
  model select offers.

### Effort vocabulary — resolved

**One effort vocabulary, four values: `low`, `medium`, `high`, `xhigh`.** This is not a new value set —
it is the one already load-bearing across the codebase: `ModelTier` (`apps/desktop/src/types/core.ts`),
`harness::models::tier_fallback` (`crates/locus-core/src/harness/models.rs`), and
`migrations/0009_model_tier_settings`. `minimal`, which appears only in the Harnesses mockup's band
table, is stale fixture prose — the same category of error `design-revision` Decision 2 already named
and corrected for the Telemetry facet — and is retired.

**Complexity and effort are different axes.** `ComplexityBand` (`xtra-low … max`, six values) sizes a
task; effort states how hard the model should think once sized. They are not 1:1 — a band's `effort`
field is drawn from the four-value set, and adjacent bands commonly share a value (`xtra-low` and `low`
both plausibly carry `effort: low`; `xtra-high` and `max` both plausibly carry `effort: xhigh`). The
mapping is per-band and per-harness, set in the Harnesses band table, never fixed.

**Surfaces that change:**

- Workshop → Harnesses: the band table's Effort column offers exactly the four values, never `minimal`
  and never free text.
- `RoutingBand.effort` and `RoutingDefaults.effort` (`runtime/routing.rs`) narrow from `String` to the
  shared four-value type.
- `migrations/0017_autorouting_decisions.up.sql`'s `routing_effort` column is unconstrained free text
  today (`btrim(routing_effort) <> ''` only); its `CHECK` gains the four-value enumeration.
- The Harness record's `default effort` field and Plan → Decompose's per-task Effort override both draw
  from the same four values — Decompose was already correct and becomes the vocabulary's second
  canonical surface, not a second vocabulary.

### Providers

- **List** — a status dot: `ok`, `warn`, `off`. This is a three-state UI projection over a two-state
  `VerificationStatus` (`Verified` / `Failed` in `crates/locus-core/src/services/provider.rs`) plus
  staleness: `ok` is a recent `Verified`, `off` is `Failed` or never verified, and `warn` — the mockup's
  "token expires in 6 days" case — has no backing field yet and is a gap this feature closes by adding
  an expiry/staleness signal the status derives from, not a stored third enum state.
- **Authentication** — one credential per provider, every harness pointed at it shares it. Method
  segmented `OAuth / API key / None` (`ProviderConnectionConfig::authentication_method`, already
  modeled). Masked secret with **Reveal** and **Replace**, a `keychain` tag naming the reference. **The
  database stores only `core.providers.keychain_reference` — no migration under `migrations/` has ever
  added a secret-bearing column, and none may.** `base_url` override for a proxy or gateway
  (`ProviderConnectionConfig::base_url`, already modeled). A verify line reporting the last check and
  model count, or a warning.
- **Preferred models** — `Model · Alias · Context · In/out per M · In selector · (remove)`, backed by
  `core.provider_models` and `services::provider::selector_projection`. An alias is what the model
  selector shows for every harness pointed at this provider; without one the selector shows the full
  id. A catalogue search reports "n of N match".
- **Right rail** — a selector preview; harnesses using this provider, with the removal contract stated
  verbatim: "Removing this provider unsets the model on each of them rather than failing their next
  run." — this is a behavior `ProjectHarnessPolicy` and the harness record must implement, not only
  describe; and a 30-day spend figure, read from the telemetry aggregate the metrics spec computes,
  not computed here.

### CLI

- **Built-in tools**, grouped by category (Source control, Search & files, Rust, Database, Web &
  network), each with per-tool toggles and a tri-state group master
  (`ToolGroupEnablement::from_tools`, already modeled). "Off means it is not in the image at all" —
  `ToolCatalog::enabled_image_set` already excludes a disabled tool from the image; the UI states the
  existing invariant rather than introducing one.
- **Uploaded tools require a valid Minisign signature before admission.** `ToolCatalog::admit_user_tool`
  keeps its existing fail-closed behavior: an invalid, missing, or untrusted signature means the tool
  never enters the catalog or an image. The mockup's read-only-role alternative is rejected: image
  construction executes the uploaded install path, so a read-only agent role does not contain that
  risk. The UI explains the rejection and offers signing; it never displays an unsigned tool as usable.
- **Dropzone** — a binary, script, or `tool.toml`, or an install line (`cargo install`, `npm -g`,
  `pipx`, a release URL); `install` and `verify-it-landed` fields; pinned by digest
  (`ToolManifest::binary_sha256`, already verified against the uploaded binary).
- **Image card** — "Enabled tools are baked into the base image, not installed per run. A change
  rebuilds it once." — size and last rebuild. **Most reached for**, from telemetry tool-call counts.

### Workflows

Header: editable title, a **Visual / Governance** switch, an autosave chip (`saved 2s ago`), an
**Inspector** toggle. No Save, no Validate — a workflow definition is either being edited live or it
is not open; there is no separate commit step to skip. List grouped **Published / Draft / Archived**,
each row carrying node count, edit time, and references (`referenced by 1 schedule`).

**Visual** — palette **Agent, Task, Loop, Condition, Gate, Verify** (`Verify` tagged *required*).
Presets expand into ordinary, editable nodes rather than configuring an opaque block. **There is no
Goal node** — this supersedes `.specs/workflow-canvas/spec.md`'s node vocabulary, where `Goal` is
listed as an on-canvas approval gate; Goal moves to Governance (below) as the guiding statement and the
run's termination condition, not a node with handles and edges.

The condition inspector offers an expression builder, the compiled expression, and a validity chip:
"total · evaluable in the core · reproducible from stored events." Operands are the enumerated set
`verify.passed`, `verify.exit_code`, `iteration`, `elapsed`, `tokens.used`,
`events.count(tool_error)`, `events.last(kind)`, `artifact.exists(kind)`, `task.status`,
`mail.pending` — under the rule **"No code, no model, no I/O — anything this cannot express is a
Gate."** `crates/locus-core/src/services/workflow.rs` has no operand enumeration yet; this feature adds
one so the inspector's chip list and the core's validator read the same source rather than two lists
drifting apart.

**Governance** — three sections, backed by `WorkflowGovernance` (`services/workflow.rs`), which already
carries `goal`, `guardrails`, and `success_criteria` as one versioned, immutable unit:

- **Goal** — the guiding statement every node is judged against, and the termination condition: the
  loop exits when the goal is met, not when the agent says it is finished. Can link to a plan instead
  of free text.
- **Guardrails** — titled markdown prompt cards, read by the run while in flight, and re-injected into
  context after any reset — not only at the first iteration.
- **Success criteria** — `(check) · Kind · Criterion · Checked by`; kinds `command / assertion /
  human`; checked by `core` or `gate — you`. **A criterion the core cannot check itself becomes a
  gate**: it goes to the inbox with the evidence attached, rather than being marked passed on the
  agent's word. This is an escalation `services/workflow.rs` must express structurally — a
  `SuccessCriterionKind::Human` criterion has no core checker and its evaluation path is a gate, not an
  optional branch.

**The authoring surface holds no run state.** `governance_is_versioned` in `services/workflow.rs`
already asserts serialized Governance carries no `execution` or `results` key; this feature extends
that same assertion to the Visual graph's serialized form.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `desktop-workshop-runtime` | this spec, in full |
| `workflow-canvas` — node vocabulary only (the `Goal` node) | this spec's Workflows → Visual contract; `workflow-canvas`'s graph-validation, live-overlay, and role-contamination contract stands unchanged |

Every spec superseded by an M0.7 feature carries a pointer line to its replacement, per
`design-revision`'s Acceptance 4.

## Acceptance

1. All nine extension-editor types render from one shared three-pane component; Harnesses is the only
   type whose right rail has no Materialization block.
2. The extension editor's Save is manual on every type in that group; Workflows shows an autosave chip
   and no Save control anywhere in its header.
3. The frontmatter field-kind inference table is shared code, not per-type duplication — one field
   named `budget_tokens` and one named `tools` on two different types resolve through the same
   function.
4. No surface in the product renders the value `minimal` for effort; `low`, `medium`, `high`, `xhigh`
   are the only effort values offered on the Harnesses band table, the Harness record's default effort,
   and Plan → Decompose.
5. `RoutingBand.effort` and `RoutingDefaults.effort` are a four-value type, not `String`; a value
   outside the four fails to construct.
6. `routing_effort` in `agents.runs` is constrained to the four values at the database level.
7. A band with `model_id: None` still falls to the next band up — `AutoroutingPolicy::route`'s existing
   behavior is unchanged by the type narrowing.
8. No harness is selectable without a registered adapter — `ProjectHarnessPolicy::select` still returns
   `AdapterUnavailable` for an adapter-less harness, and the Harnesses UI never offers one.
9. `core.providers` and every table added by this feature store a keychain reference only; no column
   anywhere holds a raw secret.
10. A disabled built-in tool is absent from `ToolCatalog::enabled_image_set`.
11. An uploaded tool with a missing, invalid, or untrusted signature is rejected before catalog or image
    admission; the UI offers signing rather than a read-only-role exception.
12. The Workflows palette offers exactly six node kinds — Agent, Task, Loop, Condition, Gate, Verify —
    and no Goal node, in the palette component and in `.specs/workflow-canvas/spec.md`'s node table.
13. A `SuccessCriterionKind::Human` criterion has no core-checked path; its only evaluation route is a
    gate carrying evidence.
14. Serialized Governance and the serialized Visual graph both carry no `execution` or `results` key.

## Open

- The Providers `warn` status needs a staleness or expiry signal `ProviderVerificationMetadata` does
  not carry today (it holds `verified_at` and a two-state `status`, not an expiry). This feature adds
  the signal; the exact staleness window (mirroring the mockup's "6 days") is a product default set
  during implementation, not fixed by this spec.
- The 30-day spend figure on Providers reads an aggregate the metrics spec computes. This spec
  defines only its consumption on the Providers right rail, not the aggregate itself.
- Adapter config (`Key · Value · Type`) has no `locus-core` struct today; this feature ships it as a
  JSONB blob read by the adapter. Whether it earns a typed schema is deferred until a second adapter
  needs config the free-form table cannot express.
