import {
  CONTRADICTIONS,
  GRAPH_EDGES,
  GRAPH_NODES,
  INGEST_LOG,
  KIND_COUNTS,
  LINKS,
  LINT_FINDINGS,
  PAGES,
  SELECTED_PAGE_ID,
} from '../fixtures/wiki'
import type { GraphNode, LintFinding } from '../fixtures/wiki'
import type { Contradiction, IngestLogEntry, PageKind, WikiLink, WikiPage } from '../types/wiki'

export { GRAPH_CAPTION, INGEST_NOTE, LINT_CLEAN_LINE, MEMORY_DISTINCTION, PAGE_KINDS } from '../fixtures/wiki'
export type { GraphNode, LintFinding } from '../fixtures/wiki'

/** Becomes: invoke("wiki_pages") */
export function useWikiPages(): WikiPage[] {
  return PAGES
}

/** Becomes: invoke("wiki_pages", { kind }) — grouped client-side; the kinds are fixed. */
export function useWikiPagesByKind(kind: PageKind): WikiPage[] {
  return PAGES.filter((p) => p.kind === kind)
}

/** Becomes: invoke("wiki_kind_counts") */
export function useWikiKindCounts(): Record<PageKind, number> {
  return KIND_COUNTS
}

/** Becomes: invoke("wiki_page", { id }) */
export function useWikiPage(id: string): WikiPage | null {
  return PAGES.find((p) => p.id === id) ?? null
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultPageId(): string {
  return SELECTED_PAGE_ID
}

/** Becomes: invoke("wiki_graph") */
export function useWikiLinks(): WikiLink[] {
  return LINKS
}

/** Becomes: invoke("wiki_graph", { pageId }) */
export function useWikiGraph(): { nodes: GraphNode[]; edges: Array<{ from: string; to: string }> } {
  return { nodes: GRAPH_NODES, edges: GRAPH_EDGES }
}

/** Becomes: invoke("wiki_contradictions") */
export function useContradictions(): Contradiction[] {
  return CONTRADICTIONS
}

/** Becomes: invoke("wiki_lint") */
export function useWikiLint(): LintFinding[] {
  return LINT_FINDINGS
}

/** Becomes: invoke("wiki_ingest_log") */
export function useIngestLog(): IngestLogEntry[] {
  return INGEST_LOG
}
