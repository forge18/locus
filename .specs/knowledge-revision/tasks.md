# knowledge-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Fact revision schema: `rev`, written-by run, curated-by human | — | `cargo test -p locus-core memory::fact_revision_schema` |
| 2 | Editing a fact appends revision 2; revision 1 is retained unmodified | 1 | `cargo test -p locus-core memory::edit_appends_revision` |
| 3 | Recall returns the latest revision; its provenance still resolves to the run that wrote revision 1 | 2 | `cargo test -p locus-core memory::recall_returns_curated` |
| 4 | Confidence state as a closed enum: `verified`, `asserted`, `decaying`, `contradicted` | — | `cargo test -p locus-core memory::confidence_state_enum` |
| 5 | A `contradicted` fact carries no score | 4 | `cargo test -p locus-core memory::contradicted_has_no_score` |
| 6 | `locus memory adjudicate` resolves a write-time contradiction row | 4 | `cargo test -p locus-cli memory::adjudicate` |
| 7 | `locus memory explain <id>` transcript command | 3 | `cargo test -p locus-cli memory::explain` |
| 8 | Short-term sessions rail: live sessions with resident-token readouts, near-ceiling flag | — | `pnpm -C apps/desktop test -- memory/short-term-sessions-rail` |
| 9 | Resident-now table in fixed prefix order: base-context, rules, skills, plan, recalled facts, tool results, assistant turns | 8 | `pnpm -C apps/desktop test -- memory/short-term-resident-order` |
| 10 | Resident-now rows carry a size and a `cached` / `re-read` / `volatile` tag, plus the four-fifths-tool-output reading | 9 | `pnpm -C apps/desktop test -- memory/short-term-tags-and-reading` |
| 11 | Compacted-out list: `tool · description · size → artifact id`, each id opening the artifact | 9 | `pnpm -C apps/desktop test -- memory/short-term-compacted-out` |
| 12 | Prefix-cache percentage and its invalidation-rule copy | 9 | `pnpm -C apps/desktop test -- memory/short-term-cache-copy` |
| 13 | "What survives the iteration" panel: facts, artifacts, the plan and its checked steps — not the reasoning | 9 | `pnpm -C apps/desktop test -- memory/short-term-survives` |
| 14 | Ceiling stats: compaction threshold, per-result compaction trigger | 9 | `pnpm -C apps/desktop test -- memory/short-term-ceiling-stats` |
| 15 | Long-term confidence badges for the four states | 4,5 | `pnpm -C apps/desktop test -- memory/long-term-confidence-badges` |
| 16 | Confidence sparkline with the verified-jump annotation | 15 | `pnpm -C apps/desktop test -- memory/long-term-sparkline` |
| 17 | Fact detail: locator, author, promotion time, recall count, "Why this is trusted" | 15 | `pnpm -C apps/desktop test -- memory/long-term-fact-detail` |
| 18 | Curation editor: editing writes revision 2, both revisions shown, recall marked as returning the curated one | 2,17 | `pnpm -C apps/desktop test -- memory/long-term-curation-editor` |
| 19 | Contradiction card at write, with the two values, their sources, and Adjudicate wired to the core verb | 6,15 | `pnpm -C apps/desktop test -- memory/long-term-contradiction-card` |
| 20 | Decay stats panel: below-threshold count, promoted-on-verify count, median age of a decaying fact | 15 | `pnpm -C apps/desktop test -- memory/long-term-decay-stats` |
| 21 | `locus memory explain` transcript rendered in the right rail | 7 | `pnpm -C apps/desktop test -- memory/long-term-explain-transcript` |
| 22 | Scope locked to the project — no project switcher, "never cross-project" label | 15 | `pnpm -C apps/desktop test -- memory/long-term-scope-locked` |
| 23 | Wiki kind filter — All, Decisions, Concepts, Entities, Sources, Syntheses — with counts | — | `pnpm -C apps/desktop test -- wiki/kind-filter` |
| 24 | Page detail: revision, assertion and source counts, links out, provenance | 23 | `pnpm -C apps/desktop test -- wiki/page-detail` |
| 25 | Graph mini-panel importing the canvas renderer, not duplicating it | 23 | `pnpm -C apps/desktop test -- wiki/graph-mini-panel` |
| 26 | Contradiction card at ingest, offering Adjudicate and Board card | 23 | `pnpm -C apps/desktop test -- wiki/contradiction-card-ingest` |
| 27 | `locus wiki lint` panel: orphans, broken links, unnamed entities, unsourced assertions, clean count | 23 | `pnpm -C apps/desktop test -- wiki/lint-panel` |
| 28 | Artifacts left rail split: Review artifacts vs Reference · never in the inbox | — | `pnpm -C apps/desktop test -- artifacts/rail-split` |
| 29 | Per-kind viewer dispatch: diff, walkthrough, image, recording, diagram, finding, payload | 28 | `pnpm -C apps/desktop test -- artifacts/viewer-dispatch` |
| 30 | The same artifact id renders identically from Short-term, a session record, and this screen | 11,29 | `pnpm -C apps/desktop test -- artifacts/three-entry-points` |
| 31 | Comment composer: live indicator, Send to session, Resolve | 29 | `pnpm -C apps/desktop test -- artifacts/comment-composer` |
| 32 | Mail three-pane frame, tabs All / Waiting / To you | — | `pnpm -C apps/desktop test -- mail/three-pane-tabs` |
| 33 | Thread status vocabulary — waiting, open, replied, you, drained — rendered per row | 32 | `pnpm -C apps/desktop test -- mail/thread-status-vocabulary` |
| 34 | Mail-wait banner with "State is `waiting`, not idle. The idle guardrail will not fire." verbatim | 33 | `pnpm -C apps/desktop test -- mail/wait-banner-copy` |
| 35 | Messages carry their verb tag: `mail send`, `mail read`, `mail reply`, `mail wait` | 33 | `pnpm -C apps/desktop test -- mail/verb-tags` |
| 36 | Composer actions Drain and Unblock, wired to the mail verbs | 35 | `pnpm -C apps/desktop test -- mail/drain-unblock` |
| 37 | Handoff boundary panel: drafted handoff artifact, ownership-transfer copy | 35 | `pnpm -C apps/desktop test -- mail/handoff-boundary` |
| 38 | Participants rail: run ids and states, "why you can read this" note | 32 | `pnpm -C apps/desktop test -- mail/participants-rail` |
