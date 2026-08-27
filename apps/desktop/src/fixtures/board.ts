// schema: board.tasks + board.dependency_edges + board.evidence_links
// replaced by: invoke("board_tasks") + emit("task_moved")

import type { BoardColumn, DependencyEdge, Task } from "../types/board";

/**
 * The second column's label. Settled: **In Progress**, as PLAN.md §The board
 * names it. The handoff drew "Building"; the architecture wins, and the screen
 * follows the architecture rather than the mockup.
 */
export const SECOND_COLUMN_LABEL = "In Progress";
/** What the handoff drew, kept so the difference stays legible against the PNGs. */
export const SECOND_COLUMN_ALTERNATIVE = "Building";

export const HEADER_NOTE = "Fixed columns across every project";
export const BLOCKED_NOTE = "blocked is a status, not a column";
export const APPROVAL_NOTE = "an inbox item, not a place to go looking";

export const TASKS: Task[] = [
 {
  id: "t-001",
  projectId: "p-tapestry",
  repoId: "r-tapestry-app",
  title: "Notification channel behind the Sink trait",
  column: "ready",
  status: "ok",
  verifyCommand: "cargo test -p tapestry-core notify::",
  assignee: null,
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-002",
  projectId: "p-loom-db",
  repoId: "r-loom-db",
  title: "Online index rebuild",
  column: "ready",
  status: "blocked",
  verifyCommand: "cargo test -p loom-db index::",
  assignee: null,
  gate: "human",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-003",
  projectId: "p-weaver",
  repoId: "r-weaver",
  title: "Drop the legacy parser branch",
  column: "ready",
  status: "ok",
  verifyCommand: "cargo test -p weaver parser::",
  assignee: null,
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-004",
  projectId: "p-tapestry",
  repoId: "r-tapestry-app",
  title: "Thread the channel through Supervisor::spawn",
  column: "in_progress",
  status: "ok",
  verifyCommand: "cargo test -p tapestry-core supervisor::",
  assignee: "builder@4",
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
  ciStatus: "passed",
  ciLog: "build #184 passed",
 },
 {
  id: "t-005",
  projectId: "p-weaver",
  repoId: "r-weaver",
  title: "Parser: recover from an unterminated block",
  column: "in_progress",
  status: "stuck",
  verifyCommand: "cargo test -p weaver parser::",
  assignee: "builder@4",
  gate: "reviewer agent",
  stuckIterations: 3,
  maxIterations: 3,
  tools: "full tools",
  tokens: "102.3k",
  ciStatus: "failed",
  ciLog: "build #183 failed · parser::unterminated",
 },
 {
  id: "t-006",
  projectId: "p-texere",
  repoId: "r-texere",
  title: "Ingest: dedupe on sha256 rather than path",
  column: "testing",
  status: "ok",
  verifyCommand: "pnpm test -- ingest",
  assignee: "builder@3",
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-007",
  projectId: "p-tapestry",
  repoId: "r-tapestry-web",
  title: "Retry policy on the payments client",
  column: "reviewing",
  status: "ok",
  verifyCommand: "pnpm test -- payments/retry",
  assignee: "reviewer@2",
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-008",
  projectId: "p-loom-db",
  repoId: "r-loom-db",
  title: "Vacuum schedule for the events table",
  column: "reviewing",
  status: "ok",
  verifyCommand: "cargo test -p loom-db vacuum::",
  assignee: "reviewer@2",
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-009",
  projectId: "p-tapestry",
  repoId: "r-tapestry-app",
  title: "Notification plan",
  column: "waiting_for_approval",
  status: "ok",
  verifyCommand: "cargo test -p tapestry-core notify::",
  assignee: "planner@3",
  gate: "human",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
 },
 {
  id: "t-010",
  projectId: "p-weaver",
  repoId: "r-weaver-docs",
  title: "Regenerate the CLI reference",
  column: "done",
  status: "ok",
  verifyCommand: "pnpm -C weaver-docs test",
  assignee: "ingest@2",
  gate: "reviewer agent",
  stuckIterations: null,
  maxIterations: 3,
  tools: "read-only tools",
  tokens: null,
  externalLink: "https://github.com/forge18/locus/issues/42",
  completionStatus: "resolved",
  completionAttempts: 1,
  resolutionSupported: true,
 },
];

/** Fixed columns, in board order. There is no add-column affordance. */
export const COLUMN_ORDER: BoardColumn[] = [
 "ready",
 "in_progress",
 "testing",
 "reviewing",
 "waiting_for_approval",
 "done",
];

export const COLUMN_LABELS: Record<BoardColumn, string> = {
 ready: "Ready",
 in_progress: SECOND_COLUMN_LABEL,
 testing: "Testing",
 reviewing: "Reviewing",
 waiting_for_approval: "Waiting For Approval",
 done: "Done",
};

export const DEPENDENCIES: DependencyEdge[] = [
 { fromTaskId: "t-001", toTaskId: "t-004" },
 { fromTaskId: "t-004", toTaskId: "t-009" },
];

/** Evidence is what makes Done mean something: runs, events, artifacts. */
export const EVIDENCE: Record<string, { runs: number; events: number }> = {
 "t-010": { runs: 2, events: 41 },
};
