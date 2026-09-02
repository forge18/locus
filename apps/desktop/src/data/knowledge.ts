import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";
import type {
  CompactedContext,
  FactConfidence,
  KnowledgeFact,
  ResidentLayer,
} from "./demo/fixtures/knowledge";

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
} from "./demo/fixtures/knowledge";
export type {
  CompactedContext,
  FactConfidence,
  KnowledgeFact,
  ResidentLayer,
  ResidentTag,
} from "./demo/fixtures/knowledge";

/** Live project-scoped long-term facts. */
export function fetchLongTermFacts(
  projectId?: string,
): Promise<Envelope<KnowledgeFact[]>> {
  return dataProvider().query<KnowledgeFact>("memory_facts", { projectId });
}

export interface MemoryMutationReceipt {
  updated: boolean;
}

/** Adjudication changes only the confidence state; fact revisions remain append-only. */
export function setMemoryFactConfidence(
  projectId: string,
  factId: string,
  confidence: FactConfidence,
): Promise<Envelope<MemoryMutationReceipt>> {
  return dataProvider().queryOne<MemoryMutationReceipt>(
    "memory_confidence_set",
    {
      projectId,
      factId,
      confidence,
    },
  );
}

/** Becomes: invoke('memory_short_term') */
export function useResidentLayers(): ResidentLayer[] {
  return dataProvider().read?.<ResidentLayer[]>("memory_short_term") ?? [];
}

/** Becomes: invoke('memory_compacted_artifacts') */
export function useCompactedContext(): CompactedContext[] {
  return (
    dataProvider().read?.<CompactedContext[]>("memory_compacted_artifacts") ??
    []
  );
}

/** Becomes: invoke('memory_facts', { projectId }) */
export function useLongTermFacts(): KnowledgeFact[] {
  return dataProvider().read?.<KnowledgeFact[]>("memory_facts") ?? [];
}

/** Every contradicted fact intentionally has no score. */
export function factScore(factId: string): number | null {
  return (
    dataProvider()
      .read?.<KnowledgeFact[]>("memory_facts")
      ?.find((fact) => fact.id === factId)?.score ?? null
  );
}
