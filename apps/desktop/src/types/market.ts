// Mirrors the `market` Postgres schema (PLAN.md §Data model): manifests, installs,
// and the per-image tool sets an install implies.

/** @schema market — what a marketplace entry offers. */
export type ManifestKind = 'agent' | 'skill' | 'workflow' | 'linter' | 'toolset'

/** @schema market — one published entry. */
export interface Manifest {
  id: string
  name: string
  kind: ManifestKind
  version: string
  source: string
  description: string
  /** Tools the entry needs present in the image. */
  requiresTools: string[]
}

/** @schema market — an entry installed into this Locus. */
export interface Install {
  id: string
  manifestId: string
  version: string
  installedAt: string
  enabled: boolean
}

/** @schema market — what tools a given container image actually carries. */
export interface ImageToolSet {
  image: string
  tools: string[]
}
