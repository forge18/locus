import { METRICS, PROJECT_ROWS, RUNS_BY_HOUR, WANTS_ATTENTION } from '../fixtures/status'
import type { AttentionRow, HourBar, Metric, ProjectRow } from '../fixtures/status'

export type { AttentionKind, AttentionRow, HourBar, Metric, ProjectRow } from '../fixtures/status'

/** Becomes: invoke("status_metrics") */
export function useStatusMetrics(): Metric[] {
  return METRICS
}

/** Becomes: invoke("runs_by_hour") */
export function useRunsByHour(): HourBar[] {
  return RUNS_BY_HOUR
}

/** Becomes: invoke("wants_attention") */
export function useWantsAttention(): AttentionRow[] {
  return WANTS_ATTENTION
}

/** Becomes: invoke("projects_table") */
export function useProjectRows(): ProjectRow[] {
  return PROJECT_ROWS
}
