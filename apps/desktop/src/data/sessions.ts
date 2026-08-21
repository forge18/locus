import {
  RUNS,
  SELECTED_DETAIL_ID,
  SELECTED_SESSION_ID,
  SESSIONS,
  SESSION_DETAILS,
  eventsFor,
} from '../fixtures/sessions'
import type { SessionDetail } from '../fixtures/sessions'
import type { AgentEvent } from '../types/event'
import type { Run, Session } from '../types/agents'

export {
  GUARDRAIL_NOTE,
  HANDOFF_SUMMARY,
  PTY_NOTE,
  SESSION_LIST_FOOTER,
  WAITING_NOTE,
} from '../fixtures/sessions'
export type { SessionDetail, TranscriptLine } from '../fixtures/sessions'

/** Becomes: invoke("sessions_list") */
export function useSessions(): Session[] {
  return SESSIONS
}

/** Becomes: invoke("session", { id }) */
export function useSession(id: string): Session | null {
  return SESSIONS.find((s) => s.id === id) ?? null
}

/** Becomes: invoke("runs_for_session", { sessionId }) */
export function useRunsForSession(sessionId: string): Run[] {
  return RUNS.filter((r) => r.sessionId === sessionId)
}

/** Becomes: Channel<AgentEvent>("session_events", { sessionId }) */
export function useSessionEvents(sessionId: string): AgentEvent[] {
  return eventsFor(sessionId)
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultSessionId(): string {
  return SELECTED_SESSION_ID
}

/** How badly a session wants a person. Higher goes first. */
function attention(s: SessionDetail): number {
  if (s.status === 'stuck') return 3
  if (s.status === 'waiting') return 2
  if (s.status === 'idle') return 1
  return 0
}

/**
 * Becomes: invoke("sessions_list", { detail: true })
 *
 * Sorted needs-attention first, then activity — the same rule the strip uses,
 * because it is the same question: what needs a person, and what moved recently.
 */
export function useSessionDetails(): SessionDetail[] {
  return [...SESSION_DETAILS].sort(
    (a, b) => attention(b) - attention(a) || a.idleMinutes - b.idleMinutes,
  )
}

/** Becomes: invoke("session", { id, detail: true }) */
export function useSessionDetail(id: string): SessionDetail | null {
  return SESSION_DETAILS.find((s) => s.id === id) ?? null
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultDetailId(): string {
  return SELECTED_DETAIL_ID
}
