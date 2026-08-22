import { invoke } from '@tauri-apps/api/core'
import { RECENTLY_EDITED, TYPE_CARDS } from '../fixtures/extensions'
import type { EditedEntry, TypeCard } from '../fixtures/extensions'

export {
  CACHE_READ_RATE,
  DETERMINISM_NOTE,
  ENTRY_TYPE,
  HEADER_NOTE,
  HEADER_TITLE,
  NEW_LABEL,
  SEARCH_PLACEHOLDER,
} from '../fixtures/extensions'
export type { EditedEntry, TypeCard } from '../fixtures/extensions'

export const LINTERS_ROOT = '/locus/config/linters'

/** Read the one extension type Locus owns directly rather than a harness. */
export async function fetchLinterCountFromCore(): Promise<number> {
  return invoke<number>('linter_count', { root: LINTERS_ROOT })
}

/** Becomes: invoke("extension_inventory") */
export function useTypeCards(): TypeCard[] {
  return TYPE_CARDS
}

/** Becomes: invoke("recently_edited") */
export function useRecentlyEdited(): EditedEntry[] {
  return RECENTLY_EDITED
}
