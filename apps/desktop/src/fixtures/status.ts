// schema: agents.sessions + agents.runs + workflows.verify_results + core.projects
// replaced by: invoke("status_metrics") + invoke("projects_table")

export interface Metric {
  label: string
  /** Null where the number is not knowable — it renders *unknown*, never 0. */
  value: string | null
  /** Unit suffix, set at 15px beside the numeral. */
  unit: string | null
  note: string | null
  /** The one card that means "you are the blocker" gets the accent treatment. */
  attention: boolean
  /** A note that belongs in --status-danger rather than --text-secondary. */
  badNote: string | null
}

export const METRICS: Metric[] = [
  { label: 'Running', value: '8', unit: null, note: '4 panes · 4 strip', attention: false, badNote: null },
  { label: 'Waiting on me', value: '3', unit: null, note: 'oldest 26m', attention: true, badNote: null },
  { label: 'Verify pass', value: '71', unit: '%', note: 'last 24h', attention: false, badNote: null },
  { label: 'Cache read', value: '84', unit: '%', note: 'of input tokens', attention: false, badNote: null },
  { label: 'Tokens today', value: '4.2', unit: 'M', note: '3 harnesses report nothing', attention: false, badNote: null },
  { label: 'Guardrail trips', value: '5', unit: null, note: null, attention: false, badNote: '1 kill & reassign' },
]

export interface HourBar {
  hour: string
  passed: number
  failed: number
  aborted: number
}

/** Twelve stacked bars, one per hour, newest last. */
export const RUNS_BY_HOUR: HourBar[] = [
  { hour: '08', passed: 6, failed: 1, aborted: 0 },
  { hour: '09', passed: 9, failed: 2, aborted: 1 },
  { hour: '10', passed: 11, failed: 1, aborted: 0 },
  { hour: '11', passed: 7, failed: 4, aborted: 1 },
  { hour: '12', passed: 4, failed: 1, aborted: 0 },
  { hour: '13', passed: 8, failed: 2, aborted: 0 },
  { hour: '14', passed: 12, failed: 3, aborted: 2 },
  { hour: '15', passed: 10, failed: 2, aborted: 0 },
  { hour: '16', passed: 6, failed: 5, aborted: 1 },
  { hour: '17', passed: 9, failed: 1, aborted: 0 },
  { hour: '18', passed: 5, failed: 2, aborted: 1 },
  { hour: '19', passed: 3, failed: 0, aborted: 0 },
]

/** waiting is not idle: one is blocked on a gate, the other on nothing at all. */
export type AttentionKind = 'stuck' | 'idle' | 'waiting'

export interface AttentionRow {
  kind: AttentionKind
  subject: string
  detail: string
  action: string | null
}

export const WANTS_ATTENTION: AttentionRow[] = [
  { kind: 'stuck', subject: 'weaver · builder@4', detail: 'stuck 3/3 · 102.3k tokens', action: 'Reassign' },
  { kind: 'idle', subject: 'loom-db · builder@4', detail: 'idle 3m · no event on the stream', action: null },
  { kind: 'waiting', subject: 'texere · builder@3', detail: 'waiting: gate — not idle', action: null },
]

export interface ProjectRow {
  project: string
  repos: number
  running: number
  inReview: number
  /** Verify pass rate; colored --status-success or --status-danger by the screen, not stored coloured. */
  verify: number
  tokensToday: string | null
  /** Null where the harness reports no usage: cache read is unknown, not 0%. */
  cache: string | null
  lastEvent: string
}

export const PROJECT_ROWS: ProjectRow[] = [
  { project: 'tapestry', repos: 2, running: 3, inReview: 1, verify: 78, tokensToday: '1.71M', cache: '88%', lastEvent: '2s ago' },
  { project: 'loom-db', repos: 1, running: 2, inReview: 0, verify: 74, tokensToday: '1.09M', cache: '91%', lastEvent: '3m ago' },
  { project: 'weaver', repos: 3, running: 2, inReview: 1, verify: 44, tokensToday: '0.98M', cache: '61%', lastEvent: '8s ago' },
  // texere runs on a harness that reports no usage, so both its spend and its
  // cache-read rate are unknown — neither is zero.
  { project: 'texere', repos: 1, running: 1, inReview: 2, verify: 69, tokensToday: null, cache: null, lastEvent: '41s ago' },
]
