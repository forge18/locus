// M0.7 knowledge fixtures. These are intentionally UI-shaped until Tauri
// commands replace them; identifiers remain stable so views can share handles.

export const SHORT_TERM_COPY = {
  intro: 'The context window. Nothing here is stored — it is rebuilt from scratch every iteration, which is why what goes in it is a design decision and not a cache.',
  residentNote: 'in prefix order — the order is the cache, so it never varies',
  residentReading: 'Four fifths of the window is tool output. Everything authored… is under 4k, which is why the prefix stays cached and why an unstable materialization is expensive out of proportion to its size.',
  compactedNote: 'written to an artifact, replaced by one line naming it',
  compactedReading: 'Compaction is the bridge between the two memories: short-term drops it, and it becomes an artifact the agent can fetch again by name. Nothing is lost, only moved.',
  cacheNote: 'Stable while the materialized tree is stable. A reordered extension invalidates the prefix for every run that follows it, not just the next one.',
  survivesReasoning: 'Everything else, including the reasoning',
} as const

export type ResidentTag = 'cached' | 're-read' | 'volatile'

export interface ResidentLayer {
  name: string
  size: string
  percent: string
  tag: ResidentTag
}

/** Fixed prefix order is part of the cache contract. */
export const RESIDENT_LAYERS: ResidentLayer[] = [
  { name: 'base-context', size: '1.2k', percent: '14%', tag: 'cached' },
  { name: 'rules in scope', size: '0.8k', percent: '9%', tag: 'cached' },
  { name: 'skills loaded', size: '1.8k', percent: '21%', tag: 'cached' },
  { name: 'the live plan', size: '0.6k', percent: '7%', tag: 're-read' },
  { name: 'recalled facts', size: '0.9k', percent: '11%', tag: 're-read' },
  { name: 'tool results', size: '31.4k', percent: '76%', tag: 'volatile' },
  { name: 'assistant turns', size: '4.5k', percent: '11%', tag: 'volatile' },
]

export interface CompactedContext {
  tool: string
  description: string
  size: string
  artifactId: string
}

export const COMPACTED_CONTEXT: CompactedContext[] = [
  { tool: 'web_fetch', description: 'agentclientprotocol.com/protocol', size: '62.4kB', artifactId: 'a-7802' },
  { tool: 'bash', description: 'cargo build — full output', size: '18.1kB', artifactId: 'a-7811' },
  { tool: 'read_file', description: 'store/mod.rs — whole file', size: '9.7kB', artifactId: 'a-7815' },
]

export type FactConfidence = 'verified' | 'asserted' | 'decaying' | 'contradicted'

export interface KnowledgeFact {
  id: string
  title: string
  score: number | null
  confidence: FactConfidence
  recall: string
}

export const LONG_TERM_FACTS: KnowledgeFact[] = [
  { id: 'mem-1184', title: 'NOTIFY payload caps at 8000 bytes', score: 0.94, confidence: 'verified', recall: 'recalled 31×' },
  { id: 'mem-1098', title: 'Partition key must be in the primary key', score: 0.88, confidence: 'verified', recall: 'recalled 12×' },
  { id: 'mem-1044', title: 'AppKit eats the cmd-chord before JS sees it', score: 0.61, confidence: 'asserted', recall: 'recalled 4×' },
  { id: 'mem-1007', title: 'Port range is 43000–43999', score: null, confidence: 'contradicted', recall: 'no score' },
  { id: 'mem-1011', title: 'sqlx offline mode needs a prepared cache', score: 0.44, confidence: 'decaying', recall: 'last recall 19d' },
  { id: 'mem-1072', title: 'Verify runs in a fresh container, never the agent’s', score: 0.91, confidence: 'verified', recall: 'recalled 22×' },
]

export const CURATION_COPY = 'Editing this makes it yours, not the agent’s. The page keeps both: the written fact stays as rev 1, your correction becomes rev 2, and recall returns the curated one while provenance still points at the run.'

export const WIKI_KIND_CHIPS = [
  { kind: 'all', label: 'All', count: 153, definition: '…the only place the wiki can be checked as a whole: the link graph, the orphans, the assertions with no source.' },
  { kind: 'decision', label: 'Decisions', count: 14, definition: 'A fork, the option taken, and the cost of taking it. The only page kind that closes an argument.' },
  { kind: 'concept', label: 'Concepts', count: 31, definition: 'An idea the codebase assumes. Named here so an agent can be told it once instead of inferring it every run.' },
  { kind: 'entity', label: 'Entities', count: 42, definition: 'A thing the system has: a daemon, a table, a container. Orphans are flagged, because an entity nothing links to is usually a rename nobody finished.' },
  { kind: 'source', label: 'Sources', count: 57, definition: 'What was ingested, verbatim and unedited. Every assertion elsewhere points back to one of these.' },
  { kind: 'synthesis', label: 'Syntheses', count: 8, definition: 'An answer assembled from several pages that exists nowhere on its own. The only kind an agent writes unprompted.' },
] as const

export const WIKI_INGEST_COPY = 'Derived, then curated — a path or a URL, never a blank page.'
export const WIKI_GRAPH_COPY = 'Pages are nodes, wikilinks are edges — the canvas renderer, repointed.'
export const WIKI_CONTRADICTION_COPY = 'flagged at ingest, not at query'
export const MEMORY_DISTINCTION_COPY = 'The wiki is curated prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else.'
