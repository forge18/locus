# knowledge-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision` · **Blocks** M3 coordination, memory and mail

## Purpose

What an agent knows, made inspectable across its four homes: the context window that is rebuilt every
iteration, the facts that survive a run, the prose a human curates, and the messages agents exchange
while working. `memory` and `wiki` describe the mechanism; this feature is the screen contract that
makes it legible, and it closes the one semantic question `memory` left open — what an edit does to a
recalled fact.

Mail gets a screen for the first time. It was mechanics only until now: a CLI verb set with no way to
read a thread, watch a `mail wait` timeout, or see a handoff form.

## Governed by

- PLAN.md §Memory — the four layers, capture, injection, promotion, decay, recall, the keeper
- PLAN.md §The wiki — typed pages, ingest, contradictions, the linter, the graph
- PLAN.md §Knowledge, as one model — why memory and the wiki share `pgvector` and nothing else
- PLAN.md §Artifacts — what you review instead of tool calls, the review/reference split
- PLAN.md §The user inbox — the inbox is you as an addressee in the mail system
- PLAN.md §Token discipline — prefix stability, and summaries with handles, never bodies
- `docs/UI_MOCKUP_REVIEW.md` §Memory → Short-term, §Memory → Long-term, §Memory → Wiki,
  §Memory → Artifacts, §Mail — the reviewed contract this spec does not restate

## Contract

### Short-term

The context window as an anatomy, not a cache: "Nothing here is stored — it is rebuilt from scratch
every iteration." **Resident now** lists in **fixed prefix order** — base-context, rules in scope,
skills loaded, the live plan, recalled facts, tool results, assistant turns — because "the order is the
cache, so it never varies." Each row carries a size and a tag: `cached`, `re-read`, `volatile`. The
reading this order exists to support: four fifths of the window is tool output, everything authored is
under 4k, so an unstable materialization is expensive out of proportion to its size.

**Compacted out** is the bridge to the second memory: each row shows `tool · description · size →
artifact id`. "Short-term drops it, and it becomes an artifact the agent can fetch again by name.
Nothing is lost, only moved" — the same rule `tool-compaction` and `artifacts` already enforce; this
screen is where it becomes visible.

The right rail carries the prefix-cache percentage and its invalidation rule ("a reordered extension
invalidates the prefix for every run that follows it, not just the next one"), **What survives the
iteration** (facts written to long-term, artifacts, the plan and its checked steps — not the
reasoning), and ceiling stats (compaction threshold, per-result compaction trigger).

### Long-term

Facts carry one of four confidence states: **verified**, **asserted**, **decaying**, **contradicted**.
A contradicted fact shows no score. **Provenance beats recency** — a passing verify outranks any later
assertion, shown as **Why this is trusted**: the verify that confirmed it, the pages citing it, recall
frequency. A confidence sparkline reads `asserted 0.38 → verified 0.94 · the jump is the verify, not
the repetition`.

**Curation semantics — closes the open question in `.specs/memory`.** Editing a recalled fact does not
overwrite it. The written fact stays as revision 1; the correction becomes revision 2. Recall returns
the curated revision (the latest), while provenance on that revision still points at the run that wrote
revision 1. Editing makes the fact yours, not the agent's, without discarding what the agent originally
asserted or where it came from.

A **contradiction card** is flagged at write time, carrying both conflicting values and their sources,
with **Adjudicate**. Decay statistics (fell below recall threshold, promoted on a passing verify, median
age of a decaying fact) sit beside a `locus memory explain` transcript. Scope is locked per project —
labeled **never cross-project** — matching the project-scoped promotion boundary in `.specs/memory`.

### Wiki

A kind filter — **All / Decisions / Concepts / Entities / Sources / Syntheses** — with each kind's
definition shown verbatim:

| Kind | Definition |
| --- | --- |
| Decisions | "A fork, the option taken, and the cost of taking it. The only page kind that closes an argument." |
| Concepts | "An idea the codebase assumes. Named here so an agent can be told it once instead of inferring it every run." |
| Entities | "A thing the system has: a daemon, a table, a container. Orphans are flagged, because an entity nothing links to is usually a rename nobody finished." |
| Sources | "What was ingested, verbatim and unedited. Every assertion elsewhere points back to one of these." |
| Syntheses | "An answer assembled from several pages that exists nowhere on its own. The only kind an agent writes unprompted." |
| All pages | "…the only place the wiki can be checked as a whole: the link graph, the orphans, the assertions with no source." |

**Ingest, not authoring** — the primary action is "Ingest a document"; a GUI editor exists but is not
the entry point. Page detail carries revision, assertion and source counts, ingest and curation times,
**Links out** as wikilink chips, and **Provenance** listing each source. A graph mini-panel is "the
canvas renderer, repointed" — pages as nodes, wikilinks as edges.

The contradiction card here is flagged **at ingest, not at query**, offering **Adjudicate** and **Board
card** — distinct from Long-term's write-time card, which offers Adjudicate only. A `locus wiki lint`
panel reports orphans, broken links, entities mentioned but never given a page, and assertions with no
source.

### Artifacts

The left rail splits **Review artifacts** (comment counts, may reach the inbox) from **Reference ·
never in the inbox** (findings and payloads — storage with a handle). Kinds: `diff`, `walkthrough`,
`image`, `recording`, `diagram`, `finding`, `payload`. One viewer per kind; the same artifact renders
identically from its three entry points (Short-term's compacted-out row, a session's own record, and
this screen).

Comments steer: a thread mixes human comments and agent replies, shows a live indicator when the run is
still going, and a composer offers **Send to session** / **Resolve** — routing into the live session per
`.specs/artifacts`.

### Mail

Three panes. Left: tabs **All / Waiting / To you**, threads carrying project, status, subject,
`from → to`. Status vocabulary: **waiting**, **open**, **replied**, **you**, **drained**.

Center: the thread, headed by a **mail-wait banner** when a participant is blocked in `mail wait` —
`builder@4 is in mail wait — 8m of a 15m timeout` — with the invariant stated verbatim: **"State is
`waiting`, not idle. The idle guardrail will not fire."** Messages carry their verb tag: `mail send`,
`mail read`, `mail reply`, `mail wait`. The composer replies as the human, with **Drain** and
**Unblock**.

Right: participants with their run ids and states ("Different containers, one address space. Mail
survives a harness swap mid-project."); the verb set `send / read / reply / wait / drain`; **the
handoff boundary** — "The moment ownership transfers it stops being mail and becomes a handoff, with a
payload the successor reads instead of this thread" — with the drafted handoff artifact when one
exists; and **why the thread is readable at all** — "Agent-to-agent mail is stored, not ephemeral. When
a run goes wrong the question is usually what one agent told another — and it was invisible until
here."

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `desktop-knowledge-review` | this spec, for the Memory group (Short-term, Long-term, Artifacts, Wiki); the Inbox half moves to `shell-revision`, the hands-on half is replaced by `interact-sessions` |
| `screens-wiki` | this spec — `screens-wiki` already points at `design-desktop`, itself superseded by `design-revision` |

Mail as a screen has no predecessor: `.specs/mail` covers the CLI and the fold, never a view.

## Acceptance

1. Short-term's Resident-now list renders in the fixed prefix order — base-context, rules, skills,
   plan, recalled facts, tool results, assistant turns — regardless of window contents.
2. Every Compacted-out row resolves to a fetchable artifact by id; nothing it names is discarded.
3. A fact carries exactly one of `verified` / `asserted` / `decaying` / `contradicted`; a contradicted
   fact renders with no score.
4. **Editing a recalled fact writes revision 2 and keeps revision 1 unchanged; recall returns revision 2
   while its provenance still resolves to the run that wrote revision 1.**
5. A contradiction raised at write time on Long-term and one raised at ingest time on Wiki are distinct
   cards — Wiki's alone offers Board card.
6. The Wiki kind filter shows six chips (All plus the five named kinds); no chip is labeled `overview`.
7. `locus wiki lint` findings (orphan, broken link, unnamed entity, unsourced assertion) each render in
   the lint panel with the clean count.
8. A `finding` or `payload` artifact never appears in an inbox query result.
9. The same artifact id renders identically opened from Short-term, a session record, and the Artifacts
   screen.
10. An artifact comment composed on this screen reaches the live session that produced it.
11. A thread's status cell is always one of `waiting` / `open` / `replied` / `you` / `drained`.
12. While a participant's state is `waiting`, the idle guardrail does not fire for that run.
13. Ownership transfer on a thread produces a handoff artifact and the thread stops accepting new mail
    verbs.

## Open

- **Wiki page-kind discrepancy.** `.specs/wiki` names six kinds — `source`, `entity`, `concept`,
  `synthesis`, `decision`, `overview` — with `overview` a living synthesis revised on every ingest. The
  mockup's kind filter shows only five plus All: Decisions, Concepts, Entities, Sources, Syntheses.
  `overview` has no chip and no visible page in the reviewed contract. Either the mockup drops a kind
  the backend still writes, or `overview` folds into `synthesis` for display; `.specs/wiki`'s own open
  question about per-ingest regeneration cost makes this worth resolving before `wiki` ships rather than
  after.
- Whether Long-term's Adjudicate and Wiki's Adjudicate write to the same resolution mechanism, or two
  that happen to share a name — both close a contradiction row, but Long-term's is a memory fact and
  Wiki's can additionally raise a board card.
