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

export interface PersistedExtension {
  id: string;
  extensionType: string;
  name: string;
  version: number;
  frontmatter: Record<string, unknown>;
  body: string;
  updatedAt: string;
}

export interface ExtensionRevision {
  id: string;
  extensionId: string;
  version: number;
  frontmatter: Record<string, unknown>;
  body: string;
  createdAt: string;
}

export interface ExtensionSaveRequest {
  id?: string;
  extensionType: string;
  name: string;
  frontmatter: Record<string, unknown>;
  body: string;
}

/** Load the current authored extension rows from Postgres in live mode. */
export function fetchExtensions(
  extensionType: string,
): Promise<import("./envelope").Envelope<PersistedExtension[]>> {
  return dataProvider().query<PersistedExtension>("extensions_list", {
    extensionType,
  });
}

/** Save one authored extension and append an immutable revision. */
export function saveExtension(
  request: ExtensionSaveRequest,
): Promise<import("./envelope").Envelope<PersistedExtension>> {
  return dataProvider().queryOne<PersistedExtension>("extension_save", {
    request,
  });
}

/** Load immutable revisions for the History panel. */
export function fetchExtensionHistory(
  extensionId: string,
): Promise<import("./envelope").Envelope<ExtensionRevision[]>> {
  return dataProvider().query<ExtensionRevision>("extension_history", {
    extensionId,
  });
}

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
