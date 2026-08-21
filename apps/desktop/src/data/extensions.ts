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

/** Becomes: invoke("extension_inventory") */
export function useTypeCards(): TypeCard[] {
  return TYPE_CARDS
}

/** Becomes: invoke("recently_edited") */
export function useRecentlyEdited(): EditedEntry[] {
  return RECENTLY_EDITED
}
