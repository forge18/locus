// Mirrors the `mail` Postgres schema (PLAN.md §Data model): threads, messages,
// and delivery state.

/** @schema mail — a conversation between agents, or between an agent and you. */
export interface Thread {
  id: string
  subject: string
  /** Agent names, or "human". */
  participants: string[]
  messageIds: string[]
  updatedAt: string
}

/** @schema mail — whether a message reached its recipient. */
export type DeliveryState = 'queued' | 'delivered' | 'read' | 'failed'

/**
 * @schema mail — one message. The body is a summary and a handle, never a payload:
 * an artifact is referenced by id, not inlined.
 */
export interface Message {
  id: string
  threadId: string
  from: string
  to: string[]
  body: string
  /** Artifacts referenced by handle. */
  artifactIds: string[]
  state: DeliveryState
  sentAt: string
}
