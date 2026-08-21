// Mirrors the `memory` Postgres schema (PLAN.md §Data model): core memory,
// bounded and materialized per run, and the store of facts with provenance.

/**
 * @schema memory — bounded, per-agent and per-project, materialized fresh into
 * every run. Bounded is the point: it is a prompt prefix, not an archive.
 */
export interface CoreMemory {
  id: string
  projectId: string
  /** Null for a project-wide block. */
  agent: string | null
  body: string
  /** The cap this block is held under, in characters. */
  budget: number
  updatedAt: string
}

/** @schema memory — where a fact is allowed to apply. */
export type MemoryScope = 'global' | 'project' | 'repo' | 'agent'

/**
 * @schema memory — one fact, with where it came from and how much it is trusted.
 * Confidence decays, so a fact nobody reconfirms fades rather than hardening.
 */
export interface MemoryFact {
  id: string
  scope: MemoryScope
  scopeId: string | null
  body: string
  /** The run or session that produced it. */
  provenance: string
  confidence: number
  decayAt: string
  createdAt: string
}
