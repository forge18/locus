// Mirrors the `agents` Postgres schema's normalized event rows
// (PLAN.md §Canonical event vocabulary). Every telemetry source normalizes to
// exactly this set — there is no thirteenth verb.

/** @schema agents — the twelve canonical verbs. Every source normalizes to these. */
export const EVENT_VERBS = [
  'session_start',
  'user',
  'assistant',
  'thinking',
  'tool_call',
  'tool_result',
  'tool_error',
  'permission_request',
  'subagent_start',
  'subagent_stop',
  'aborted',
  'session_end',
] as const

/** @schema agents — one of the twelve canonical verbs. */
export type EventVerb = (typeof EVENT_VERBS)[number]

/**
 * @schema agents — token usage exactly as the harness reported it. Locus never
 * counts tokens itself, so where a harness reports nothing this is null and spend
 * reads *unknown*. It is never a zero.
 */
export interface Usage {
  input: number
  output: number
  cacheRead: number
  cacheWrite: number
}

/**
 * @schema agents — one normalized event. `seq` is assigned by the core on arrival,
 * so a source with no ordering guarantee still yields a totally ordered stream.
 * `raw` keeps the source record, so a normalization bug is repairable by replay.
 */
export interface AgentEvent {
  id: string
  runId: string
  seq: number
  ts: string
  verb: EventVerb
  /** Message text for `user`, `assistant`, `thinking`. */
  text?: string
  /** Tool name for `tool_call`, `tool_result`, `tool_error`. */
  tool?: string
  /** Tool arguments, already separated from the name by the hook path. */
  args?: Record<string, unknown>
  /** Carried on `assistant` and `session_end`. Null means the harness said nothing. */
  usage?: Usage | null
  raw: Record<string, unknown>
}
