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

import { dataProvider } from './provider'
import type { Envelope } from './envelope'

/** Live count of human-pending deliveries — the Inbox pill (slice 4). The full
 * Inbox list migration is slice 7; these fixture accessors stay until then. */
export async function fetchInboxPendingCount(): Promise<Envelope<number>> {
  return dataProvider().queryOne<number>('inbox_pending_count')
}
