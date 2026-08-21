// Mirrors the `wiki` Postgres schema (PLAN.md §Data model): pages typed by kind,
// revisions, links, contradictions, the ingest log, and pgvector embeddings.

/**
 * @schema wiki — a page's kind decides how it is written and how it is trusted.
 * Six, matching the tree groups the wiki screen is drawn with.
 */
export type PageKind =
  | 'overview'
  | 'decision'
  | 'concept'
  | 'entity'
  | 'synthesis'
  | 'source'

/** @schema wiki — curated prose a human reads, derived by ingest then cleaned up. */
export interface WikiPage {
  id: string
  projectId: string
  title: string
  kind: PageKind
  /** Prose, one entry per paragraph. Backticked spans are inline paths. */
  body: string[]
  /** Current revision number. */
  revision: number
  /** How many claims the page makes, and how many sources back them. */
  assertions: number
  sources: number
  ingestedAgo: string
  curatedAgo: string
  /** Slugs this page points at, as they appear inside [[wikilinks]]. */
  links: string[]
  /** Where the page came from: an icon name and a line. */
  provenance: Array<{ icon: string; line: string }>
  /** True where nothing links in. The tree flags it and the linter counts it. */
  orphan: boolean
}

/** @schema wiki — one prior version of a page. */
export interface Revision {
  id: string
  pageId: string
  revision: number
  body: string
  at: string
  by: string
  summary: string
}

/** @schema wiki — a directed link between two pages. */
export interface WikiLink {
  fromPageId: string
  toPageId: string
}

/**
 * @schema wiki — two pages that cannot both be true. Recorded rather than resolved,
 * because which one is wrong is a human call.
 */
export interface Contradiction {
  id: string
  pageIds: [string, string]
  claim: string
  detectedAt: string
  resolved: boolean
  /**
   * Both statements and both sources. Neither is optional: a flag a reader cannot
   * adjudicate from what is on screen is an alarm, not a finding.
   */
  values: [ContradictionSide, ContradictionSide]
  /** When it was caught. Ingest time, not query time. */
  note: string
}

/** @schema wiki — one side of a contradiction: what was said, and by what. */
export interface ContradictionSide {
  value: string
  source: string
  age: string
}

/** @schema wiki — what ingest read, and what it produced from it. */
export interface IngestLogEntry {
  id: string
  source: string
  pagesWritten: number
  at: string
  status: 'ok' | 'failed'
  note: string | null
}
