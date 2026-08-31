import type { Envelope } from './envelope'
import { dataProvider } from './provider'

/** Wire type: one pending human delivery — the Inbox list's item. */
export interface InboxDelivery {
  id: string
  threadId: string
  subject: string
  body: string
  senderKind: string
  project: string
  createdAt: string | null
}

/** Wire type: one delivery drained today — the RESOLVED TODAY list's row. */
export interface ResolvedDelivery {
  id: string
  subject: string
  body: string
  project: string
  resolvedAt: string | null
}

/** Wire type: the Inbox pill's and header's real counts. */
export interface InboxThroughput {
  pending: number
  resolvedToday: number
}

/** Live read: every human-pending delivery, newest first. The host scopes by
 * project when a projectId is given; an unknown project is a typed not-found. */
export function fetchInboxList(
  projectId?: string,
): Promise<Envelope<InboxDelivery[]>> {
  return dataProvider().query<InboxDelivery>('inbox_list', { projectId })
}

/** Live read: deliveries drained today, newest first. */
export function fetchResolvedToday(
  projectId?: string,
): Promise<Envelope<ResolvedDelivery[]>> {
  return dataProvider().query<ResolvedDelivery>('inbox_resolved_today', {
    projectId,
  })
}

/** Live read: the pending and resolved-today counts. */
export function fetchInboxThroughput(): Promise<Envelope<InboxThroughput>> {
  return dataProvider().queryOne<InboxThroughput>('inbox_throughput')
}

/** Drain a delivery. An empty comment resolves silently; a comment is recorded
 * on the thread as the human's decision. */
export function resolveInboxDelivery(
  deliveryId: string,
  comment: string,
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>('inbox_resolve', {
    deliveryId,
    comment,
  })
}

/** Live count of human-pending deliveries — the Inbox pill (slice 4). */
export async function fetchInboxPendingCount(): Promise<Envelope<number>> {
  return dataProvider().queryOne<number>('inbox_pending_count')
}
