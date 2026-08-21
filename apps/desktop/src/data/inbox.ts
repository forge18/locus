import { INBOX_ITEMS, RESOLVED_TODAY } from '../fixtures/inbox'
import type { InboxItem, ResolvedItem } from '../fixtures/inbox'

export type { InboxItem, InboxKind, ResolvedItem } from '../fixtures/inbox'

/** Becomes: invoke("inbox_list") + emit("inbox_changed") */
export function useInboxItems(): InboxItem[] {
  return INBOX_ITEMS
}

/** Becomes: invoke("inbox_resolved_today") */
export function useResolvedToday(): ResolvedItem[] {
  return RESOLVED_TODAY
}
