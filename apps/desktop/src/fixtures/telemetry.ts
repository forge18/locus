// schema: agents.events + agents.runs (aggregates) + workflows.verify_results
// replaced by: invoke("telemetry_aggregates") + invoke("telemetry_facets")

import { EVENT_VERBS } from '../types/event'
import type { EventVerb } from '../types/event'
import { rng } from './rng'

export const SEARCH_QUERY = 'tool_error'
export const SEARCH_NOTE = 'every event, every session · BM25 over the normalized log'

export interface FilterChip {
  label: string
  active: boolean
}

export const FILTER_CHIPS: FilterChip[] = [
  { label: 'verify: failed', active: true },
  { label: '30d', active: true },
]

export const RESET_LABEL = 'Reset filters'

export interface TelemetryMetric {
  label: string
  value: string
  unit: string | null
  /** Set on the one metric that is a problem rather than a measurement. */
  bad: boolean
}

export const METRICS: TelemetryMetric[] = [
  { label: 'Sessions', value: '641', unit: null, bad: false },
  { label: 'Events', value: '154,385', unit: null, bad: false },
  { label: 'Tool errors', value: '2,190', unit: null, bad: true },
  { label: 'Output tokens', value: '77.46', unit: 'M', bad: false },
]

/** Sixteen bars, one per interval. Sessions over time. */
export const SPARKLINE: number[] = (() => {
  const next = rng(1616)
  return Array.from({ length: 16 }, () => Math.round(next() * 70) + 25)
})()

export interface Facet {
  value: string
  count: number
  active: boolean
  /** Dimmed where the count is zero *by design* rather than by chance. */
  invariant?: boolean
}

export interface FacetGroup {
  key: string
  label: string
  facets: Facet[]
}

/**
 * The branch group states an invariant rather than a filter: Locus never works in
 * `main`, and a facet showing `main 0` proves it every time the page loads. A
 * sentence claiming the same thing proves nothing.
 */
export const FACET_GROUPS: FacetGroup[] = [
  {
    key: 'harness',
    label: 'harness',
    facets: [
      { value: 'claude', count: 412, active: false },
      { value: 'codex', count: 88, active: false },
      { value: 'cursor', count: 41, active: false },
      { value: 'gemini', count: 26, active: false },
      { value: 'aider', count: 14, active: false },
    ],
  },
  {
    key: 'capture_source',
    label: 'capture source',
    facets: [
      { value: 'hooks', count: 505, active: false },
      { value: 'acp', count: 41, active: false },
      { value: 'stream-json', count: 53, active: false },
      { value: 'session-log', count: 72, active: false },
    ],
  },
  {
    key: 'project',
    label: 'project',
    facets: [
      { value: 'tapestry', count: 288, active: false },
      { value: 'loom-db', count: 163, active: false },
      { value: 'weaver', count: 122, active: false },
      { value: 'texere', count: 68, active: false },
    ],
  },
  {
    key: 'agent_role',
    label: 'agent · role',
    facets: [
      { value: 'builder@4', count: 201, active: false },
      { value: 'reviewer@2', count: 140, active: false },
      { value: 'engineer@1', count: 98, active: false },
      { value: 'auditor@2', count: 71, active: false },
      { value: 'keeper@1', count: 33, active: false },
    ],
  },
  {
    key: 'model_tier',
    label: 'model tier',
    facets: [
      { value: 'high', count: 334, active: false },
      { value: 'medium', count: 180, active: false },
      { value: 'low', count: 45, active: false },
      { value: 'xhigh', count: 18, active: false },
    ],
  },
  {
    key: 'verify',
    label: 'verify',
    facets: [
      { value: 'failed', count: 126, active: true },
      { value: 'passed', count: 484, active: false },
      { value: 'aborted', count: 15, active: false },
    ],
  },
  {
    key: 'arbiter_class',
    label: 'arbiter class',
    facets: [
      { value: 'bug', count: 61, active: false },
      { value: 'spec gap', count: 34, active: false },
      { value: 'noise', count: 21, active: false },
      { value: 'ambiguity', count: 12, active: false },
    ],
  },
  {
    key: 'branch',
    label: 'branch',
    facets: [
      { value: 'agent/*', count: 641, active: false },
      // Zero by invariant, not by chance. Locus never works in main.
      { value: 'main', count: 0, active: false, invariant: true },
    ],
  },
]

export interface ActionRow {
  verb: EventVerb
  count: number
  /** Drawn in --bad rather than accent. */
  bad: boolean
  /**
   * Set on `permission_request`. It is a misconfiguration alarm, not a metric —
   * one firing means a harness gate was left on and the run is about to hang.
   */
  alarm: string | null
}

export const ACTION_NOTE = 'the canonical vocabulary — every source normalizes to it'

export const ACTION_ROWS: ActionRow[] = [
  { verb: 'tool_call', count: 50_796, bad: false, alarm: null },
  { verb: 'tool_result', count: 48_708, bad: false, alarm: null },
  { verb: 'assistant', count: 47_837, bad: false, alarm: null },
  { verb: 'thinking', count: 11_264, bad: false, alarm: null },
  { verb: 'user', count: 5_611, bad: false, alarm: null },
  { verb: 'tool_error', count: 2_190, bad: true, alarm: null },
  { verb: 'subagent_start', count: 1_318, bad: false, alarm: null },
  { verb: 'subagent_stop', count: 1_311, bad: false, alarm: null },
  { verb: 'session_start', count: 641, bad: false, alarm: null },
  { verb: 'session_end', count: 626, bad: false, alarm: null },
  { verb: 'aborted', count: 15, bad: true, alarm: null },
  {
    verb: 'permission_request',
    count: 2,
    bad: true,
    alarm:
      '2 permission_requests is a misconfiguration alarm, not a metric — a harness launched with its own gate on. Both runs hung.',
  },
]

export const MISSING_VERB_NOTE =
  'A missing verb is recorded as missing, never synthesized: thinking is absent for the 9 session-log runs.'

export interface ToolRow {
  tool: string
  count: number
}

export const TOOL_NOTE = 'payload by tool, from the arbiter'

export const TOOL_ROWS: ToolRow[] = [
  { tool: 'bash', count: 26_092 },
  { tool: 'read_file', count: 9_303 },
  { tool: 'edit_file', count: 8_689 },
  { tool: 'rg+grep', count: 4_970 },
  { tool: 'locus memory', count: 3_314 },
  { tool: 'cargo nextest', count: 2_572 },
  { tool: 'locus artifact', count: 1_886 },
  { tool: 'web_fetch', count: 1_119 },
  { tool: 'locus mail', count: 671 },
  { tool: 'locus agent', count: 528 },
  { tool: 'sqlx-cli', count: 479 },
  { tool: 'locus debug', count: 286 },
  { tool: 'locus ask', count: 141 },
  { tool: 'locus browse', count: 92 },
]

export const TOOL_ANOMALY =
  'Anomaly: researcher@1 ran web_fetch 4.1× its own baseline on 19 Aug — already a query, not new instrumentation.'

export interface SessionRow {
  when: string
  harness: string
  project: string
  repo: string
  agent: string
  role: string
  models: string
  runs: number
  events: number
  errors: number
  /** Null where the harness reported no usage. */
  tokens: string | null
  status: 'running' | 'stuck' | 'closed' | 'waiting' | 'aborted' | 'handed off'
  statusDetail: string | null
  id: string
}

export const SESSION_ROWS: SessionRow[] = [
  { when: '2026-08-20 09:35', harness: 'claude', project: 'tapestry', repo: 'core', agent: 'builder@4', role: 'impl', models: 'claude-opus-5', runs: 3, events: 1_402, errors: 18, tokens: '41.2k', status: 'running', statusDetail: null, id: '9cd39051' },
  { when: '2026-08-20 08:44', harness: 'claude', project: 'weaver', repo: 'desktop', agent: 'builder@4', role: 'impl', models: 'claude-opus-5', runs: 4, events: 3_118, errors: 23, tokens: '102.3k', status: 'stuck', statusDetail: 'stuck 3/3', id: 'a708eae2' },
  { when: '2026-08-20 08:22', harness: 'codex', project: 'loom-db', repo: 'db', agent: 'auditor@2', role: 'audit', models: 'gpt-5.2', runs: 1, events: 1_694, errors: 16, tokens: '9.4k', status: 'closed', statusDetail: null, id: 'a5abc2c9' },
  // texere runs on a harness that reports no usage: unknown, not zero.
  { when: '2026-08-20 08:02', harness: 'claude', project: 'texere', repo: 'media', agent: 'builder@3', role: 'impl', models: 'claude-opus-5', runs: 1, events: 182, errors: 0, tokens: null, status: 'waiting', statusDetail: 'waiting: gate', id: '3dc1e427' },
  { when: '2026-08-19 22:11', harness: 'cursor', project: 'weaver', repo: 'core', agent: 'builder@4', role: 'impl', models: 'claude-sonnet-5', runs: 3, events: 908, errors: 41, tokens: '58.1k', status: 'handed off', statusDetail: 'to reviewer@2', id: '5a71c003' },
  { when: '2026-08-19 19:40', harness: 'aider', project: 'loom-db', repo: 'db', agent: 'keeper@1', role: 'sweep', models: 'gemini-3-pro', runs: 2, events: 412, errors: 3, tokens: null, status: 'aborted', statusDetail: 'budget', id: '77b0aa1e' },
]

/**
 * The table is drawn at 300 sessions; the rows below are the first page. Later
 * pages are generated so the lazy path has something real to fetch.
 */
export const SESSION_TOTAL = 300

const PROJECTS = ['tapestry', 'loom-db', 'weaver', 'texere']
const HARNESSES = ['claude', 'codex', 'cursor', 'aider']
const AGENTS = ['builder@4', 'reviewer@2', 'auditor@2', 'builder@3']
const ROLES = ['impl', 'review', 'audit', 'impl']
const STATUSES: SessionRow['status'][] = ['running', 'closed', 'waiting', 'stuck', 'aborted']

/** The full 300, seeded so page two is the same page two on every read. */
export const ALL_SESSION_ROWS: SessionRow[] = [
  ...SESSION_ROWS,
  ...Array.from({ length: SESSION_TOTAL - SESSION_ROWS.length }, (_, i) => {
    const next = rng(90_000 + i)
    const p = Math.floor(next() * PROJECTS.length)
    const reportsUsage = next() > 0.18
    return {
      when: `2026-08-${String(19 - Math.floor(i / 40)).padStart(2, '0')} ${String(
        7 + (i % 12),
      ).padStart(2, '0')}:${String(i % 60).padStart(2, '0')}`,
      harness: HARNESSES[Math.floor(next() * HARNESSES.length)],
      project: PROJECTS[p],
      repo: ['core', 'db', 'desktop', 'media'][p],
      agent: AGENTS[Math.floor(next() * AGENTS.length)],
      role: ROLES[Math.floor(next() * ROLES.length)],
      models: ['claude-opus-5', 'gpt-5.2', 'claude-sonnet-5'][Math.floor(next() * 3)],
      runs: Math.floor(next() * 4) + 1,
      events: Math.floor(next() * 3_000) + 60,
      errors: Math.floor(next() * 30),
      tokens: reportsUsage ? `${(next() * 90 + 4).toFixed(1)}k` : null,
      status: STATUSES[Math.floor(next() * STATUSES.length)],
      statusDetail: null,
      id: (0xa0000000 + i).toString(16),
    }
  }),
]

export interface Facet2 {
  label: string
  value: string
  count: number
}

/** Kept for the older facet consumers; the grouped set above is the screen's. */
export const FACETS: Facet2[] = FACET_GROUPS.flatMap((group) =>
  group.facets.map((f) => ({ label: group.label, value: f.value, count: f.count })),
)

export interface VerbCount {
  verb: EventVerb
  count: number
}

export const VERB_COUNTS: VerbCount[] = EVENT_VERBS.map((verb) => ({
  verb,
  count: ACTION_ROWS.find((a) => a.verb === verb)!.count,
}))

export interface SpendRow {
  harness: string
  runs: number
  /** Null where the harness reports no usage. Spend reads *unknown*, never zero. */
  tokens: number | null
  cacheReadPct: number | null
}

export const SPEND: SpendRow[] = [
  { harness: 'claude', runs: 142, tokens: 2_410_000, cacheReadPct: 88 },
  { harness: 'codex', runs: 71, tokens: 1_120_000, cacheReadPct: 74 },
  { harness: 'cursor', runs: 44, tokens: 690_000, cacheReadPct: 61 },
  { harness: 'aider', runs: 19, tokens: null, cacheReadPct: null },
]
