import { dataProvider } from "./provider";
import type {
  ExtensionTypeCount,
  ExtensionType,
  HarnessEntry,
} from "./demo/fixtures/generated-harnesses";

export {
  EXTENSION_LABELS,
  EXTENSION_TYPES,
} from "./demo/fixtures/generated-harnesses";
export type {
  ExtensionType,
  ExtensionTypeCount,
  HarnessEntry,
  HarnessExtension,
  MechanismBadge,
} from "./demo/fixtures/generated-harnesses";

/** Becomes: invoke("harness_registry_list") */
export function useHarnesses(): readonly HarnessEntry[] {
  return dataProvider().read?.<HarnessEntry[]>("harness_registry_list") ?? [];
}

/**
 * Becomes: invoke("harness_registry_summary")
 *
 * Computed from `harnesses/*.toml`, which is why these numbers cannot go stale the
 * way the design copy's "27 of 88" did.
 */
export function useHarnessSummary(): {
  harnesses: number;
  entries: number;
  downgrades: number;
} {
  return (
    dataProvider().read?.<{
      harnesses: number;
      entries: number;
      downgrades: number;
    }>("harness_registry_summary") ?? {
      harnesses: 0,
      entries: 0,
      downgrades: 0,
    }
  );
}

/** Becomes: invoke("extension_types") */
export function useExtensionTypes(): readonly ExtensionType[] {
  return dataProvider().read?.<ExtensionType[]>("extension_types") ?? [];
}

/** Becomes: invoke("extension_inventory") */
export function useExtensionCounts(): readonly ExtensionTypeCount[] {
  return dataProvider().read?.<ExtensionTypeCount[]>("extension_counts") ?? [];
}
