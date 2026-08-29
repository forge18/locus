# Context Layer × Locus — Fit Brainstorm

2026-08-29. Companion to `RESEARCH-CONTEXT-MANAGEMENT.md` (the research artifact; kept
repo-free on purpose — this file is where the repo finally meets it). **Brainstorm, not a
spec.** `PLAN.md` remains the architecture authority; nothing here is decided. Read
`PLAN.md` §The one surface, §Memory, §Knowledge as one model, §Token discipline, and
`.specs/memory`, `.specs/tool-compaction` first — the fit is unusually tight.

---

## 1. The headline

The research synthesis (§13) described a context management layer with six components.
Locus's existing design — written before this research pass — already contains five of
them, in some places with mechanisms the literature doesn't have. The brainstorm's job is
not "import the research into locus." It is: **confirm convergence, reconcile taxonomies,
fill the four genuine gaps, and name where locus could claim the open problems.**

## 2. Convergence map — research §13 → locus

| Research component | §13 says | Locus already has | Verdict |
| --- | --- | --- | --- |
| **Store: typed, provenance, deterministic** | typed context objects; provenance; byte-deterministic serialization | memory store in Postgres with provenance, `supersedes`, path addressing; materialization is byte-deterministic by mandate (§Token discipline #1); written memory = git artifacts (a fourth layer the research didn't name) | **converged** |
| **Split-plane policy (J2 resolution)** | binding policy enforced structurally at tool boundary; explanatory policy read-only, agent-immutable | the **constitution**: human-written, small, always loaded, explicitly *"not a memory layer"* — Locus made the split-plane decision already; container is the coarse binding boundary | **converged** (binding is coarser than Progent — see §5.3) |
| **Write path: governed, quarantined, supersede-with-lineage** | hooks/governed writes; quarantine until corroborated; conflicts supersede with lineage | capture is **hooks, not tools** ("hooks log and inject; they never think"); **probation buffer** with promotion on cluster-density-of-three; dedup sets `supersedes`; re-verification against `codanna`/test runs | **converged — Locus's quarantine is more concrete than §13's** |
| **Read path: JIT with handles, sufficiency gate, placement** | handles over bodies; sufficiency-gated injection; head/tail placement; diversity guard | **800-token catalog of paths + one-liners**, bodies via `locus memory recall`; turn-level injection off for `code`/`plan` with measured justification (32.1%→25% success as k rises); mutable content last | **converged** (diversity guard implicit via k=1 — see §5.4) |
| **Assembly policy: prefix stability, breakpoints** | stable prefix + append-only tail; destructive edits at breakpoints | five prefix rules (deterministic materialization, frozen config tree, **memory catalog is a snapshot**, mutable last, no turn-level injection); cache rate is *a column with an 80% threshold*, not a project | **converged — ahead of §13** |
| **Degradation engine** | strength scores: recency + reinforcement + outcome feedback; decay in the store | Ebbinghaus curve with category half-lives × importance damping × recall reinforcement; prune at 0.05; **chain-aware pruning**; **change-driven invalidation for symbol memories** (git blame + codanna); cold-start guard | **converged — §10's "nobody ships graded decay" is wrong in locus's case** |
| **Attribution & feedback** | source attribution, paired A/B, verification cost | measured importance: *injected → recalled → verify-passed* logged per memory; `usage.cache_read` on every event; `token-optimizer top` as a GROUP BY | **partial** — memory-level yes, context-object-level no (see gap G2) |

Also already present, worth naming because the research arrived at them independently:
just-in-time retrieval with handles (Headroom CCR / OMNI pattern, §1), the summary-with-
handle rule applied to *four* surfaces (§Token discipline #4), tool-docs-on-demand
(`locus tools docs` ≈ Context7's mechanism, §1.2), and over-retrieval harm (2505.16067's
experience-following, anticipated by the k-reversal data).

## 3. Where locus is ahead of the literature

1. **Chain-aware pruning** — a decayed memory survives if any graph neighbor is strong.
   This solves the low-frequency/high-importance anticorrelation that makes naive
   Ebbinghaus pruning dangerous ("never call the production database directly"). No paper
   in the research corpus does this.
2. **Change-driven invalidation for symbol memories** — invalidating on *signature change*
   (git blame + codanna) rather than time. Decay-by-content-type: code memories die by
   diff, preferences die by curve. Nothing in §10's degradation literature is content-aware
   this way.
3. **k-reversal by task class** — retrieval depth as a per-task-class parameter with
   measured sign flips, encoded on the agent definition (`task_class: code | plan |
   research`). The sufficiency gate (2411.06037) as a *config*, not a runtime model call.
4. **Cache rate as a first-class column** — `usage.cache_read` / `usage.input` per event
   with an 80% threshold and per-run identification. §13.5 listed cache hit rate as the
   #1 metric; locus already logs it.
5. **"Hooks never think"** — deterministic passive capture instead of model-initiated
   memory tools. This sidesteps Fowler's "LLM decides to load/save" non-determinism
   entirely, and it is why locus's write path is *more* governed than §13's.

## 4. Genuine gaps — where the research adds something locus hasn't specced

- **G1 — No dynamic, capacity-conditioned assembly.** Locus's budgets are static
  thresholds (800-token catalog cap, compaction threshold shared with artifacts). The
  ContextBudget claim (§12) is that compression decisions should condition on *remaining*
  capacity, not fixed caps. Open question: does the catalog cap belong to the store-side
  (selection) while a dynamic budget governs assembly order under pressure? Ties to J1.
- **G2 — No failure attribution to context *objects*.** Locus logs which memories were
  injected and whether the run passed verify — but nothing attributes a failure to a
  specific base-context section, rule, skill, or tool description (Trace, arXiv 2608.09153).
  The event store already has the raw material; the attribution query doesn't exist.
- **G3 — No paired A/B harness for context-policy changes.** Every §13 invariant says
  context changes are regressions. `.specs/calibration-loop` is the natural home; whether
  it covers paired runs with contamination audits (JetBrains methodology) is unverified.
  The unique asset: cache-rate non-regression is assertable per change, which the JetBrains
  harness never measured.
- **G4 — Verification cost is unmeasured.** Verify pass/fail and tokens are logged; the
  *cost of establishing correctness* (2608.08709) is not. Cheap to add: the verify command's
  own duration/token cost is already in the event stream.
- **G5 — No offline consolidation job.** Promotion is event-driven (cluster of three);
  consolidation triggers at >40 catalog entries. Sleep-time-compute-style precomputation
  (anticipating queries, distilling episode clusters into summaries ahead of demand) has no
  owner. Low priority — event-driven may suffice — but the decay section's own consolidation
  trigger is reactive, not proactive.
- **G6 — Typed-context validation is partial.** Materializers guarantee determinism, and
  promotion re-verifies memory records — but there is no schema/validation pass over the
  *assembled* context the way TDS's runtime layer validates typed objects pre-serialization
  (§12). What would a "context object fails validation" even do at SessionStart? Worth a
  brainstorm, not a spec.

## 5. Tensions to reconcile (brainstorm material)

**5.1 Two taxonomies, one store.** Research §13 types objects by *role in assembly*
(instructions / evidence / memory / tool output); locus types by *knowledge kind* (code
structure / wiki / constitution / observations) and *lifetime* (working / probation /
long-term / written). These are orthogonal and both are load-bearing: assembly needs to
know where an object may sit and when it may be dropped; lifetime governs sharing and
trust. Suggestion to brainstorm: an `assembly_role` attribute on memory/context objects,
rather than re-taxonomizing. Open: does `task_class` subsume part of this?

**5.2 The working layer belongs to the sub-harness — does the promotion gate survive?**
§16's J3 resolution put a promotion gate *at breakpoints*, owned by the layer. Locus
assigns working memory to the sub-harness and captures continuously via hooks into the
probation buffer — there is no end-of-run cliff to gate. Locus's continuous capture is
strictly better than a breakpoint gate for *capture*; what the gate would have caught is
*judgment* ("this scratch note is actually a decision") — which locus defers to cluster
density and measured importance. Tension: continuous-capture + measured-importance vs
explicit promotion judgment. Probably fine; worth one thought about decisions that occur
once (density-of-three never fires).

**5.3 Policy binding is coarse.** Constitution is always-loaded context; the container is
the binding boundary; bypass-permissions is the posture. Progent-style symbolic rules
per tool call don't exist. The `PreToolUse` hook (tool-compaction spec) is the natural
enforcement point and "hooks never think" still holds — symbolic rule checks are cheap,
deterministic, and exit-0. But this is a real extension of the hook's mandate with new
failure modes (a buggy policy rule blocks every tool call). Brainstorm: is per-tool
policy enforcement worth it when the container already bounds blast radius? The leakage
paper (2608.19857) says read-scope matters even inside a container.

**5.4 Diversity guard is implicit.** k=1 for `code` handles near-duplicate injection, but
`research`-class retrieval (high k, hierarchical) has no stated guard against N
near-identical precedents — exactly where experience-following/mimicry drift shows up
(2505.16067; Manus's "don't get few-shotted"). Candidate: a diversity check in the recall
ranker, or MMR-style selection for `research` class only.

**5.5 The catalog snapshot vs long runs.** The memory catalog is frozen at SessionStart
(cache rule) — but a long run generates new memories *during* the run that the catalog
won't show until next run. Within-run recall via `locus memory recall` covers bodies, but
the catalog's path list is stale within the run. Is that acceptable (immutable prefix >
freshness), or does the next session's catalog make within-run staleness invisible? The
research says: prefix stability wins (10× arithmetic); probably accept. Worth stating as a
conscious trade rather than an accident.

**5.6 Recitation has no locus home.** The tail of locus's assembled context holds
per-run mutable values (canary, port, branch). Manus's recitation finding says the
*plan/todo* belongs at the tail too — repeatedly rewritten, deliberately cache-hostile.
Locus's workflow canvas and task orchestration own plans; nothing recites them into the
window tail. Brainstorm: does a `plan-recitation` hook/extension make sense, or does the
workflow engine's visibility make it unnecessary?

## 6. Where locus could claim the open problems (§13.6)

- **Graded decay at the window boundary**: locus's decay already drives catalog rank and
  injection — the claimed-missing mechanism exists here in embryonic form; naming and
  measuring it (does decay-ranked injection beat recency-ranked injection on task success?)
  would be a publishable delta over the §10 literature.
- **Learned assembly**: the event store + `task_class` + verify outcomes are exactly the
  training corpus MEM1-style policies need. Nobody has learned assembly against a
  verification-cost objective; locus's telemetry is closer to a corpus than anything in the
  academic set.
- **Verification-cost objective** (G4) is one SQL column away.

## 7. What the counter-evidence says locus should NOT do

- No per-turn free rebuild of the window — already honored by the frozen-snapshot rules.
- No model-initiated memory tools — already excluded ("the model must not remember to
  remember").
- No shared short-term — already forbidden (Hermes failure); matches the §13 write-path
  governance reasoning exactly.
- No headline savings claims without paired measurement — the tool-compaction spec already
  asserts *ratios*, not events; extend that discipline to any assembly-policy change.
- Don't add model calls inside hooks — the 100ms/exit-0 rules are the reason the write
  path stays trustworthy.

## 8. Open questions to carry forward (no answers attempted)

> Status after discussion 2026-08-29: resolutions in §9.

1. Does `assembly_role` earn its keep as an attribute, or do `task_class` + knowledge-kind
   already determine placement?
2. Should the calibration-loop own paired context A/Bs, and should cache-rate
   non-regression be an acceptance criterion there?
3. Is per-tool symbolic policy enforcement (Progent-style) worth extending the
   PreToolUse hook for, given container-as-boundary?
4. Where does a research-class diversity guard live — ranker, selection, or nowhere?
5. Is within-run catalog staleness (5.5) an accepted trade, documented, or does the
   catalog gain a cheap append-only "new since snapshot" section?
6. Does recitation (5.6) belong to locus at all, or to the workflow engine?
7. What would failure attribution (G2) cost over the existing event store — one view, or
   a new correlation id threaded through injection?

## 9. Resolutions from discussion (2026-08-29)

> Specified 2026-08-29: [`.specs/context-layer/spec.md`](.specs/context-layer/spec.md) +
> [`.specs/context-layer/tasks.md`](.specs/context-layer/tasks.md), TODO item 39. R1–R8 are
> the contract there; this section stays as the decision record.

**R1 — converge the taxonomies by derivation, not re-taxonomizing (5.1 / 8.1).** Note:
under the no-experiments constraint (2026-08-29), the eviction-class attribute is a
*label* for already-validated behavior (research doc §16.1), not a mechanism to be
tuned — the forced-budget ablation is retired. The two
axes are orthogonal, not competing: knowledge-kind is *authority* (who wrote it), lifetime
is *trust/sharing* (who sees it, how long it lives), and assembly-role is *where it may
sit and when it may be dropped*. Assembly-role is largely derivable from the other two
plus `task_class`. Proposal: a static derivation table in the materializer/assembly
policy — written once, every harness — mapping (knowledge-kind, lifetime, task_class) →
{placement zone, injection mechanism, eviction class}. The only per-object addition is a
three-value **eviction class** on memory records: `sticky` (decisions, corrections,
unresolved errors), `standard` (evidence), `disposable` (condensable history). The §13
five-level priority chain collapses because locus already handles tool output by
compaction threshold, not typing. Constitution and rules map to the always/never-dropped
slot outside the memory store. J1's ablation experiment then reduces to: does eviction
class ordering under forced budgets beat recency ordering — one experiment, one
attribute.

**R2 — keep continuous capture; add one declared promotion path (5.2).** Hook capture
beats a breakpoint gate for capture. The blind spot — a once-only decision that never
reaches density-of-three — is closed by a second promotion path, not a gate: `locus
memory promote` (agent or human), which skips the density check but still passes
re-verification and dedup, lands in the same probation metadata, and decays if never
recalled. Handoffs remain the human-in-loop path for session-scoped decisions; written
memory (git) remains the durable home for project decisions. No gate machinery.

**R3 — CLI-only tool surface makes coarse binding sufficient (5.3 / 8.3).** No MCP means
no third-party tool-description clash surface (Breunig's MCP clash is moot), the tool-docs
catalog is first-party, and the tool-call universe is `locus` verbs + shell commands.
Policy binding stays: container as boundary, git/branch model for scope, credential-proxy
audits for secrets. Progent-style per-tool symbolic rules are not built now; the
PreToolUse hook remains the extension point *if* a per-verb allowlist ever earns its
keep (symbolic checks are compatible with "hooks never think" — exit-0, no model). The
sufficiency gate stays relevance-only; first-party outputs need no trust tier.

**R4 — diversity guard as selection-time dedup, research class only (5.4 / 8.4).** After
BM25+vector+graph ranking, suppress candidates above an embedding-similarity threshold to
an already-selected item (MMR-lite) — only when k>1 (`research`). `code`/`plan` are
k=1 and unchanged. Cheap, ranker-local, no new objects; targets exactly the
dose-response of experience-following.

**R5 — don't unfreeze the snapshot; append freshness to the mutable tail (5.5 / 8.5).**
Accepted as written 2026-08-29; amended same day after ContextBudget verification. The
freeze *was* intentional (Hermes's frozen-snapshot mechanic, adopted for cache
stability — §Token discipline #3) and should stay: the head of the prompt is where the
10× cache arithmetic lives. Freshness belongs at the tail, which is already the mutable
zone: an **append-only "new since snapshot" catalog section** — paths + one-liners
captured during the run, small tail budget, merged into the next SessionStart snapshot.
Both properties survive: stable prefix, within-run visibility. (This resolves the
"not intentional" concern — the freeze is intentional and load-bearing; the staleness
complaint is fixed at the tail, not by unfreezing.)

**R5 amendment — the tail budget is derived and capacity-aware, not a constant.** The
user challenged the fixed tail budget ("dynamic based on the model") and asked whether
it was researched. Verification pass: the ContextBudget paper the round-2 blog cited is
now found and verified — **ContextBudget: Budget-Aware Context Management for
Long-Horizon Search Agents** (arXiv 2604.01664, April 2026): compression formulated as a
budget-constrained sequential decision problem, decisions adapting to remaining capacity;
"consistently improves reasoning robustness across a wide range of context budgets, with
the largest gains under stringent budgets" (~5× in the 32-objective regime), beating the
conservative "summarize only when full" static baseline. Consequence: the fixed tail
budget is dropped; the tail budget derives from the same effective-window derivation as
the catalog cap, and appends check remaining capacity before incorporating (plain rule,
not the paper's learned policy — that stays research-only under the no-experiments
constraint). The frozen head is untouched. R5's Open tail-budget setting is closed;
the similarity threshold remains the only Open setting.

**R6 — recitation is useful; the run supervisor owns it (5.6 / 8.6).** A one-to-three-line
recitation block at the mutable tail — current objective, current step, unresolved error
count — updated on task-state changes through the existing hook injection path (not per
turn). The workflow engine owns the plan body (fetched on demand); the tail block is a
pointer + state line, deliberately tiny. Cache cost is bounded by the tail being the
mutable zone already. Aligns with the handoffs state (§PLAN Handoffs) for cross-session
continuity.

**R7 — calibration-loop owns paired context A/Bs (8.2).** Accepted as direction:
pair-run harness with contamination audits (JetBrains methodology) plus one new
acceptance criterion the literature never had — **cache-rate non-regression** per
context-policy change, assertable from `usage.cache_read`.

**R8 — failure attribution starts as a view, not a schema change (8.7 / G2 / G4).** A
`context_attribution` view over existing events: injection events ↔ verify outcomes ↔
`tool_result` rows, plus materialization snapshots per run (which already identify
base-context/rules/skills content). Add a **verification-cost column** (G4) in the same
view — the verify command's own duration/tokens are already in the stream. Thread a
correlation id through injection only if the view later fails to disambiguate mid-run
rule loads. No new instrumentation for v1.
