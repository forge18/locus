// schema: agents.runs + agents.sessions + workflows.verify_results
// replaced by: invoke("runs_list")

import type { RunStatus } from '../types/agents'
import { ago, pick, rng } from './rng'

export const SEARCH_NOTE = 'a path, a tool name, an event verb'

/** The range control. 30d is where it opens. */
export const RANGES = [
  { value: 'today', label: 'Today' },
  { value: '7d', label: '7d' },
  { value: '30d', label: '30d' },
] as const

export const DEFAULT_RANGE = '30d'

export interface RunStat {
  label: string
  value: string
  note: string
}

/**
 * Three stats, all of them already columns. Review is a query over what the runs
 * recorded, not new instrumentation.
 */
export const RUN_STATS: RunStat[] = [
  { label: 'spec-gap rate', value: '11%', note: 'of arbiter classifications' },
  { label: 'noise reclassified', value: '21', note: 'by the arbiter, last 30d' },
  { label: 'tokens per passing run', value: '38.4k', note: 'median' },
]

export interface RunRow {
  id: string
  project: string
  agent: string
  branch: string
  status: RunStatus
  harness: string
  role: string
  /**
   * The model that actually answered — an id, never a tier. PLAN.md records it on
   * the run so spend and verify pass rate are attributable to what really ran.
   */
  model: string
  events: number
  errors: number
  /** Null where the harness reported no usage. */
  tokens: number | null
  durationSec: number
  verify: string
  at: string
}

const PROJECTS = ['tapestry', 'loom-db', 'weaver', 'texere']
const AGENTS = ['builder@4', 'builder@3', 'reviewer@2', 'planner@3', 'ingest@2']
const MODELS = ['claude-opus-5', 'claude-sonnet-5', 'gpt-5.2-codex', 'gemini-3-pro']
const HARNESSES = ['claude', 'codex', 'cursor', 'aider']
const ROLES = ['impl', 'review', 'audit', 'sweep']

/** Tier names, which must never appear in the model column. */
export const MODEL_TIERS = ['low', 'medium', 'high', 'xhigh']
const STATUSES: RunStatus[] = ['passed', 'passed', 'passed', 'failed', 'aborted']

/**
 * 612 rows, which is what the Runs table is drawn at. The count is deliberate:
 * it is the size the virtualization question gets answered against.
 */
export const RUN_ROWS: RunRow[] = (() => {
  const next = rng(612_612)
  return Array.from({ length: 612 }, (_, i) => {
    const reportsUsage = next() > 0.18
    return {
      id: `run-${String(i).padStart(4, '0')}`,
      project: pick(next, PROJECTS),
      agent: pick(next, AGENTS),
      branch: `agent/${(0x1000 + i).toString(16)}-${pick(next, ['notify', 'index', 'parser', 'ingest'])}`,
      status: pick(next, STATUSES),
      harness: pick(next, HARNESSES),
      role: pick(next, ROLES),
      model: pick(next, MODELS),
      events: Math.floor(next() * 3_000) + 40,
      errors: Math.floor(next() * 40),
      tokens: reportsUsage ? Math.floor(next() * 240_000) + 3_000 : null,
      durationSec: Math.floor(next() * 900) + 12,
      verify: pick(next, ['cargo test -p tapestry-core', 'pnpm test -- payments', 'cargo clippy --all-targets']),
      at: ago(Math.floor(next() * 2_880)),
    }
  })
})()
