// schema: agents.agent_defs (versioned; frontmatter JSONB + Markdown body)
// replaced by: invoke("agent_defs_list") + invoke("agent_def", { name })

export interface AgentDefSummary {
  name: string
  version: number
}

export const AGENT_DEFS: AgentDefSummary[] = [
  { name: 'builder', version: 4 },
  { name: 'reviewer', version: 2 },
  { name: 'interviewer', version: 3 },
  { name: 'researcher', version: 1 },
  { name: 'auditor', version: 2 },
  { name: 'keeper', version: 1 },
]

export const SELECTED_DEF = 'builder'

export const SIDEBAR_NOTE = 'Markdown plus a tool list. No canvas, no compile.'

export interface FrontmatterLine {
  key: string
  value: string
}

export const FRONTMATTER: FrontmatterLine[] = [
  { key: 'harness', value: 'claude' },
  { key: 'model_tier', value: 'high' },
  { key: 'tools', value: '[read_file, edit_file, run_command, rg]' },
  { key: 'skills', value: '[verify-loop, handoff]' },
  { key: 'rules', value: '[no-secrets, never-main]' },
  { key: 'memory_scope', value: 'project' },
]

export const PROSE: string[] = [
  'You implement one task at a time against a verify command that already exists. If there is no verify command, stop and say so — a task without one is not a task.',
  'Branch before you touch anything. Reaching main is a human action through a PR, and nothing you do gets you there.',
  'Three failing iterations on the same case is the guardrail, not a reason to try harder. Hand off with what you tried; the successor inherits the expensive half.',
]

export const PROVENANCE = 'v4 · edited 2h ago · used by 5 sessions'

export const MATERIALIZE_TARGET = '/locus/config/agents/'

/** An agent definition is immutable once a run references it. Edits make a version. */
export const NEXT_VERSION = 5
export const SAVE_LABEL = `Save as v${NEXT_VERSION}`
export const DIFF_LABEL = 'Diff v3'
