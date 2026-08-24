export type QaSchedule = 'manual' | 'push' | 'hourly' | 'daily'
export type FindingSeverity = 'fail' | 'warn'

export const QA_SCHEDULE_OPTIONS = Object.freeze([
  { value: 'manual', label: 'Manual' },
  { value: 'push', label: 'Push' },
  { value: 'hourly', label: 'Hourly' },
  { value: 'daily', label: 'Daily' },
] as const)

export interface CheckSourceDescriptor {
  id: string
  name: string
  attribution: string
  kind: string
  adapter: string
}

export const QA_CHECK_SOURCES: readonly CheckSourceDescriptor[] = Object.freeze([
  { id: 'unit-tests', name: 'Unit tests', attribution: 'vitest · cargo nextest', kind: 'tests', adapter: 'unit-tests' },
  { id: 'linters', name: 'Linters', attribution: 'clippy · eslint · ruff', kind: 'lint', adapter: 'lint-report' },
  { id: 'lsp', name: 'LSP diagnostics', attribution: 'rust-analyzer · tsserver', kind: 'lsp', adapter: 'lsp-diagnostics' },
  { id: 'agent-reviews', name: 'Agent reviews', attribution: 'reviewer@2 · custom prompt', kind: 'review', adapter: 'self-review' },
])

export interface QaFinding {
  id: string
  sourceId: string
  severity: FindingSeverity
  title: string
  project: string
  location: string
  explanation: string
  sentToInbox: boolean
}

export const QA_FINDINGS: readonly QaFinding[] = Object.freeze([
  { id: 'qa-test-1', sourceId: 'unit-tests', severity: 'fail', title: 'flaky test failed', project: 'tapestry', location: 'packages/core/src/index.test.ts:42', explanation: 'Expected an indexed result but received an empty page.', sentToInbox: false },
  { id: 'qa-lint-1', sourceId: 'linters', severity: 'warn', title: 'unused import', project: 'tapestry', location: 'apps/desktop/src/screens/dispatch/DispatchView.tsx:1', explanation: 'The import is not used by the current screen.', sentToInbox: true },
  { id: 'qa-lsp-1', sourceId: 'lsp', severity: 'warn', title: 'diagnostics unavailable', project: 'tapestry', location: 'apps/desktop', explanation: 'tsserver does not support the diagnostics verb for this workspace.', sentToInbox: false },
  { id: 'qa-review-1', sourceId: 'agent-reviews', severity: 'fail', title: 'missing verify evidence', project: 'tapestry', location: 'run-03af · artifact/diff', explanation: 'The review artifact does not include the command exit code.', sentToInbox: false },
])

export const QA_FOOTER = 'Not real-time — findings reflect the last scheduled or manual run. Sending a finding to Inbox tracks it as a to-do; it stays listed here too.'

export interface QaCheckRun {
  id: string
  project: string
  sourceId: string
  startedAt: string
  status: 'running' | 'passed' | 'failed'
}

export const QA_LAST_RUN = '4m ago'
