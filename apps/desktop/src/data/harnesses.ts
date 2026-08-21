import {
  DOWNGRADE_COUNT,
  EXTENSION_TYPES,
  ENTRY_COUNT,
  EXTENSION_COUNTS,
  HARNESSES,
  HARNESS_COUNT,
} from '../fixtures/generated/harnesses'
import type { ExtensionTypeCount, HarnessEntry } from '../fixtures/generated/harnesses'

export { EXTENSION_LABELS, EXTENSION_TYPES } from '../fixtures/generated/harnesses'
export type { ExtensionType, ExtensionTypeCount, HarnessEntry, HarnessExtension, MechanismBadge } from '../fixtures/generated/harnesses'

/** Becomes: invoke("harness_registry_list") */
export function useHarnesses(): readonly HarnessEntry[] {
  return HARNESSES
}

/**
 * Becomes: invoke("harness_registry_summary")
 *
 * Computed from `harnesses/*.toml`, which is why these numbers cannot go stale the
 * way the design copy's "27 of 88" did.
 */
export function useHarnessSummary(): {
  harnesses: number
  entries: number
  downgrades: number
} {
  return { harnesses: HARNESS_COUNT, entries: ENTRY_COUNT, downgrades: DOWNGRADE_COUNT }
}

/** Becomes: invoke("extension_types") */
export function useExtensionTypes(): readonly string[] {
  return EXTENSION_TYPES
}

/** Becomes: invoke("extension_inventory") */
export function useExtensionCounts(): readonly ExtensionTypeCount[] {
  return EXTENSION_COUNTS
}
