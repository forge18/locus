import {
  COMPACTED_CONTEXT,
  CURATION_COPY,
  LONG_TERM_FACTS,
  MEMORY_DISTINCTION_COPY,
  RESIDENT_LAYERS,
  SHORT_TERM_COPY,
  WIKI_CONTRADICTION_COPY,
  WIKI_GRAPH_COPY,
  WIKI_INGEST_COPY,
  WIKI_KIND_CHIPS,
} from "../fixtures/knowledge";

export {
  COMPACTED_CONTEXT,
  CURATION_COPY,
  LONG_TERM_FACTS,
  MEMORY_DISTINCTION_COPY,
  RESIDENT_LAYERS,
  SHORT_TERM_COPY,
  WIKI_CONTRADICTION_COPY,
  WIKI_GRAPH_COPY,
  WIKI_INGEST_COPY,
  WIKI_KIND_CHIPS,
};
export type {
  CompactedContext,
  FactConfidence,
  KnowledgeFact,
  ResidentLayer,
  ResidentTag,
} from "../fixtures/knowledge";

/** Becomes: invoke('memory_short_term') */
export function useResidentLayers() {
  return RESIDENT_LAYERS;
}

/** Becomes: invoke('memory_compacted_artifacts') */
export function useCompactedContext() {
  return COMPACTED_CONTEXT;
}

/** Becomes: invoke('memory_facts', { projectId }) */
export function useLongTermFacts() {
  return LONG_TERM_FACTS;
}

/** Every contradicted fact intentionally has no score. */
export function factScore(factId: string): number | null {
  return LONG_TERM_FACTS.find((fact) => fact.id === factId)?.score ?? null;
}
