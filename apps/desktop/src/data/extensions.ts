import { invoke, isTauri } from "@tauri-apps/api/core";
import { dataProvider } from "./provider";
import type { EditedEntry, TypeCard } from "./demo/fixtures/extensions";

export {
  CACHE_READ_RATE,
  DETERMINISM_NOTE,
  ENTRY_TYPE,
  HEADER_NOTE,
  HEADER_TITLE,
  NEW_LABEL,
  SEARCH_PLACEHOLDER,
} from "./demo/fixtures/extensions";
export type { EditedEntry, TypeCard } from "./demo/fixtures/extensions";

export const LINTERS_ROOT = "/locus/config/linters";

/** Read the one extension type Locus owns directly rather than a harness. */
export async function fetchLinterCountFromCore(): Promise<number | undefined> {
  // No host (browser preview, tests) → undefined, so the view keeps its fixture fallback.
  if (!isTauri()) return undefined;
  return invoke<number>("linter_count", { root: LINTERS_ROOT });
}

/** Becomes: invoke("extension_inventory") */
export function useTypeCards(): TypeCard[] {
  return dataProvider().read?.<TypeCard[]>("extension_inventory") ?? [];
}

/** Becomes: invoke("recently_edited") */
export function useRecentlyEdited(): EditedEntry[] {
  return dataProvider().read?.<EditedEntry[]>("recently_edited") ?? [];
}
