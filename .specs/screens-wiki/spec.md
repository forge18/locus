# screens-wiki

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · View `wiki`

## Purpose

Curated prose a human reads, derived by ingest and then cleaned up. The screen's job is to make the
distinction PLAN.md draws impossible to lose: **the wiki is not memory.** They share `pgvector` and
nothing else, and the footer says so.

The Wiki is a category in its own right with no tabs — the seventh rail item.

## Governed by

- PLAN.md §The wiki — typed pages, ingest not authoring, contradictions at ingest time, the linter
- PLAN.md §Knowledge, as one model — why the wiki is not the memory store
- `docs/design_handoff_locus_desktop_ui/README.md` screen 14

## Contract

Three panes: **246px tree · article · 284px sidebar**.

**Tree.** Primary "Ingest a document" and "Derived, then curated — a path or a URL, not a blank page."
Then typed groups with counts: overview 1, decision 14 (`gavel`), concept 31 (`lightbulb`), entity 42
(`cube`, one flagged `orphan` in `--bad`), synthesis 8, source 57 (`file-text`, `globe`). Selected page
is `--sf2` + accent ring.

**The primary action is ingest, not "New page".** A wiki nobody writes is a wiki nobody reads, so the
default path is derived-then-curated. A GUI editor still exists — a human can always fix a page — but
it is not the entry point.

**Article.** Accent kind tag + 15px title; a metadata row with mono locator, rev, assertion and source
counts, and ingest/curate ages. Prose at 13px/1.68, 88% opacity, max 720px, with mono inline paths.
`LINKS OUT` as `[[wikilink]]` pills on `--sf` + hairline. `PROVENANCE` list with icons.

**Sidebar.**
- A `GRAPH` SVG (258x132): 7px accent center node, `#0d5480` and `#314454` neighbors, hairline edges,
  8px caption, plus "Pages are nodes, wikilinks are edges — the canvas renderer, repointed."
- A `CONTRADICTIONS` card (accent ring) with two conflicting mono values, their sources, and
  "Adjudicate" / "Board card". **A contradiction carries both statements and both sources** — PLAN.md
  is explicit that a flag you cannot adjudicate yourself is just an alarm.
- `LOCUS WIKI LINT`: orphans, broken link, unnamed entities, unsourced assertion, and `--ok` "153 pages
  otherwise clean".
- Footer: "The wiki is curated prose a human reads. Memory is what an agent recalls — they share
  pgvector and nothing else."

## Acceptance

1. The tree groups pages by the six kinds, each with a count.
2. An orphan page is flagged in `--bad` in the tree, and the lint card counts it — one condition, two
   surfaces, one source.
3. The primary tree action is Ingest; no "New blank page" action is more prominent.
4. A contradiction renders both values **and** both sources, and offers both actions.
5. `[[wikilinks]]` render as pills and navigate by locator.
6. The graph SVG uses the same renderer as the workflow canvas — asserted by shared import, not by
   looking similar.
7. The wiki/memory footer renders verbatim.

## Open

- Whether wiki search lives on this screen or only in the command palette. PLAN.md gives the palette
  global search across wiki, code, tasks and runs; the handoff draws no search field here.
