// Mirrors the `workflows` Postgres schema (PLAN.md §Data model): versioned
// workflow_defs holding a graph and a spec, schedules, executions, iterations,
// guardrail trips, and verify results.

/** @schema workflows — what a node does when the execution reaches it. */
export type NodeKind = 'agent' | 'verify' | 'gate' | 'fanout' | 'join'

/** @schema workflows — one node in a workflow graph. */
export interface WorkflowNode {
  id: string
  kind: NodeKind
  label: string
  /** `agent@version` for an agent node. */
  agent: string | null
  /** The command a verify node runs. */
  verifyCommand: string | null
  x: number
  y: number
}

/** @schema workflows — a directed edge between two nodes. */
export interface WorkflowEdge {
  from: string
  to: string
  label: string | null
}

/** @schema workflows — a versioned definition: the graph, plus the spec it serves. */
export interface WorkflowDef {
  id: string
  name: string
  version: number
  nodes: WorkflowNode[]
  edges: WorkflowEdge[]
  /** The prose contract the graph implements. */
  spec: string
  updatedAt: string
}

/** @schema workflows — when a workflow runs without anyone asking. */
export interface Schedule {
  id: string
  workflowId: string
  cron: string
  enabled: boolean
  nextRunAt: string | null
}

/** @schema workflows — one execution of a workflow. */
export interface Execution {
  id: string
  workflowId: string
  startedAt: string
  endedAt: string | null
  status: 'running' | 'passed' | 'failed' | 'aborted'
  iterationIds: string[]
}

/**
 * @schema workflows — one pass through the loop. An iteration ends a run and starts
 * a new one in the same session, which is why memory and branch carry across.
 */
export interface Iteration {
  id: string
  executionId: string
  index: number
  nodeId: string
  runId: string | null
  status: 'running' | 'passed' | 'failed'
}

/**
 * @schema workflows — a guardrail firing. Three stuck iterations is kill-and-reassign,
 * which is why handoffs exist at all.
 */
export interface GuardrailTrip {
  id: string
  executionId: string
  kind: 'stuck' | 'budget' | 'timeout' | 'loop'
  action: 'warn' | 'kill_and_reassign'
  at: string
  detail: string
}

/** @schema workflows — what a verify node found. */
export interface VerifyResult {
  id: string
  iterationId: string
  command: string
  passed: boolean
  durationMs: number
  output: string
}
