# wiki

**Milestone** M5 · **Depends on** `store`, `screens-wiki`, `workflow-canvas`

## Purpose

**Ingested and typed, not a blank page.** The premise taken from `llm-wiki-agent` is the right one:
most knowledge tools make you search your own notes; this one reads everything you have collected and
writes a structured wiki that compounds. **A wiki nobody writes is a wiki nobody reads.**

## Governed by

- PLAN.md §The wiki — typed pages, ingest, contradictions, the linter, the graph
- PLAN.md §Knowledge, as one model — the wiki is not the memory store

## Contract

**Pages have kinds**, not one flat namespace:

| Kind | Holds | Created by |
| --- | --- | --- |
| `source` | one summary per ingested document | ingest |
| `entity` | a person, service, repo, or system referenced across sources | auto, on first mention |
| `concept` | an idea, pattern, or convention this project uses | auto, on first mention |
| `synthesis` | an answer to a question, filed back so it is asked once | `locus wiki query` |
| `decision` | why something is the way it is | ingest or human |
| `overview` | a living synthesis, revised on every ingest | ingest |

**Ingest, not authoring.** `locus wiki ingest <path|url>` reads a document, extracts entities and
concepts, writes or updates pages, and links them. `markitdown` handles PDF, DOCX, PPTX, XLSX, HTML.
The GUI editor still exists — a human can always fix a page — but the default path is derived then
curated.

**Contradiction flags at ingest time, not query time.** This is the idea most worth stealing: when a new
source contradicts an existing statement, the conflict is raised **when it lands** — as a row in
`wiki.contradictions` and a card on the board — rather than discovered months later by whoever happened
to read both pages.

**How it is found, and why it is bounded.** The new statement's embedding retrieves its *k* nearest
existing assertions, and **only those** go to a model to adjudicate. Ingest cost scales with what the
document says, not with how much the wiki already holds. A verdict carries **both statements and both
sources**, because a flag you cannot adjudicate yourself is just an alarm.

**The same detection serves memory:** a store-tier fact conflicting with a wiki statement is the same
problem.

**`locus wiki lint`** reports orphan pages, broken links, entities mentioned but never given a page, and
assertions with no source.

**A graph view, nearly free.** Pages are nodes, `[[wikilinks]]` are edges — the canvas renderer,
repointed. A palette, not a subsystem.

**Seeding:** every project already has written memory in git — ADRs, specs, READMEs, `AGENTS.md`. Ingest
reads those on day one so the first keeper pass has a corpus rather than a blank store.

## Acceptance

1. Ingesting a document creates a `source` page and auto-creates `entity` and `concept` pages on first
   mention.
2. `markitdown` handles a PDF, a DOCX and an HTML page.
3. Ingesting two documents that disagree produces a **contradiction card**, not two quietly conflicting
   pages.
4. A contradiction carries both statements and both sources.
5. Ingest cost scales with document size, not with wiki size — asserted by ingesting the same document
   into a small and a large wiki and comparing model calls.
6. `locus wiki lint` finds an orphan, a broken link, an unnamed entity, and an unsourced assertion.
7. The graph view imports the canvas renderer rather than duplicating it.
8. A page edited in the GUI is read back by an agent in a container.
9. Revisions are attributed to the run that made them.
10. A memory-store fact conflicting with a wiki statement raises the same contradiction.

## Open

- Whether `overview` regeneration on every ingest is affordable at scale. PLAN.md says it is revised on
  every ingest, which is a model call per document on a page that grows.
