import type { Envelope } from "./envelope";
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

export interface HarnessRegistrySummary {
    harnesses: number;
    entries: number;
    downgrades: number;
}

/** The live registry is loaded from harnesses/*.toml by Core at startup. */
export function fetchHarnesses(): Promise<Envelope<HarnessEntry[]>> {
    return dataProvider().query<HarnessEntry>("harness_registry_list");
}

export function fetchHarnessSummary(): Promise<
    Envelope<HarnessRegistrySummary>
> {
    return dataProvider().queryOne<HarnessRegistrySummary>(
        "harness_registry_summary",
    );
}

export function fetchExtensionTypes(): Promise<Envelope<ExtensionType[]>> {
    return dataProvider().query<ExtensionType>("extension_types");
}

export function fetchExtensionCounts(): Promise<
    Envelope<ExtensionTypeCount[]>
> {
    return dataProvider().query<ExtensionTypeCount>("extension_counts");
}

/** Demo/test-only synchronous accessor. Live screens use the async accessors above. */
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
    return (
        dataProvider().read?.<ExtensionTypeCount[]>("extension_counts") ?? []
    );
}
