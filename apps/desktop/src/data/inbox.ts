import { INBOX_ITEMS, INBOX_THROUGHPUT, RESOLVED_TODAY } from '../fixtures/inbox'
import type { InboxItem, InboxThroughput, ResolvedItem } from '../fixtures/inbox'

export type { InboxItem, InboxKind, InboxThroughput, ResolvedItem } from '../fixtures/inbox'

/** Becomes: invoke("inbox_list") + emit("inbox_changed") */
export function useInboxItems(): InboxItem[] {
  return INBOX_ITEMS
}

/** Becomes: invoke("inbox_resolved_today") */
export function useResolvedToday(): ResolvedItem[] {
  return RESOLVED_TODAY
}

/** Becomes: invoke("inbox_throughput") */
export function useInboxThroughput(): InboxThroughput {
  return INBOX_THROUGHPUT
}
