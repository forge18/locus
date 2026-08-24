import {
  QA_CHECK_SOURCES,
  QA_FINDINGS,
  QA_FOOTER,
  QA_LAST_RUN,
  QA_SCHEDULE_OPTIONS,
} from '../fixtures/qa'
import type { QaCheckRun, QaFinding, QaSchedule } from '../fixtures/qa'

export { QA_CHECK_SOURCES, QA_FOOTER, QA_LAST_RUN, QA_SCHEDULE_OPTIONS }
export type { CheckSourceDescriptor, FindingSeverity, QaCheckRun, QaFinding, QaSchedule } from '../fixtures/qa'

/** Becomes invoke('qa_snapshot', { projectId }). Findings are source-scoped rows. */
export function useQaFindings(projectId = 'tapestry'): QaFinding[] {
  return QA_FINDINGS.filter((finding) => finding.project === projectId).map((finding) => ({ ...finding }))
}

/** The four adapters are descriptors; adding a source does not add a name branch here. */
export function useQaSources() {
  return QA_CHECK_SOURCES
}

export function sourceFindings(findings: readonly QaFinding[], sourceId: string) {
  return findings.filter((finding) => finding.sourceId === sourceId)
}

/** A check run replaces one source's result set atomically; other sources remain intact. */
export function replaceSourceFindings(existing: readonly QaFinding[], sourceId: string, next: readonly QaFinding[]) {
  return [...existing.filter((finding) => finding.sourceId !== sourceId), ...next.filter((finding) => finding.sourceId === sourceId)]
}

export function findingSummary(findings: readonly QaFinding[]) {
  return {
    fail: findings.filter((finding) => finding.severity === 'fail').length,
    warn: findings.filter((finding) => finding.severity === 'warn').length,
  }
}

/** Manual Refresh and scheduled firings share this entry point. */
export function runQaCheck(projectId: string, sourceId: string): QaCheckRun {
  return { id: `qa-${projectId}-${sourceId}`, project: projectId, sourceId, startedAt: new Date(0).toISOString(), status: 'passed' }
}

export function sendFindingToInbox(finding: QaFinding): QaFinding {
  return { ...finding, sentToInbox: true }
}

export function scheduleLabel(schedule: QaSchedule) {
  return QA_SCHEDULE_OPTIONS.find((option) => option.value === schedule)?.label ?? 'Manual'
}
