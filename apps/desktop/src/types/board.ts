// Mirrors the `board` Postgres schema (PLAN.md §Data model): tasks with fixed
// columns, dependency edges, transitions, assignments, task/run links, evidence
// links, and linked GitHub issues.

/**
 * @schema board — the six fixed columns. There is no add-column affordance:
 * the set is the same across every project, so a task means the same thing anywhere.
 */
export const BOARD_COLUMNS = [
 "ready",
 "in_progress",
 "testing",
 "reviewing",
 "waiting_for_approval",
 "done",
] as const;

/** @schema board — one of the six fixed columns. */
export type BoardColumn = (typeof BOARD_COLUMNS)[number];

/** @schema board — blocked is a status, not a column. A blocked task keeps its place. */
export type TaskStatus = "ok" | "blocked" | "stuck";

/** @schema board — a unit of work on the board. */
export interface Task {
 id: string;
 projectId: string;
 repoId: string;
 title: string;
 column: BoardColumn;
 status: TaskStatus;
 /** The command that decides whether this task is done. */
 verifyCommand: string;
 /** `agent@version`, or null while unassigned. */
 assignee: string | null;
 /** Who or what approves it: an agent name, or "human". */
 gate: string;
 /** Set when the guardrails have counted stuck iterations against it. */
 stuckIterations: number | null;
 maxIterations: number;
 /** What the assignee is allowed to reach, as the card states it. */
 tools: string;
 /** Spend so far, or null where the harness reported none. */
 tokens: string | null;
 /** Selected workflow and owned execution summary, hydrated by Automate. */
 workflowId?: string;
 rootSessionId?: string;
 childRunIds?: string[];
 evidenceIds?: string[];
 externalLink?: string | null;
 externalHost?: string;
 /** Normalized forge check state, when the task has an external CI check. */
 ciStatus?: "pending" | "passed" | "failed";
 ciLog?: string;
 completionStatus?: "pending" | "commented" | "resolved" | "failed";
 completionAttempts?: number;
 resolutionSupported?: boolean;
 syncSupported?: boolean;
 syncState?: {
  pullCursor: string | null;
  lastPushedStatus: string | null;
  noteWatermark: string | null;
  lastLocalStatusAt: string | null;
  lastExternalStatusAt: string | null;
  lastSyncError: string | null;
  lastSyncedAt: string | null;
  unmappedExternalStatus: string | null;
  lastConflictWinner: string | null;
  lastConflictReason: string | null;
 };
 comments?: Array<{
  author: string;
  body: string;
  origin: "local" | "external";
 }>;
}

/** @schema board — a task that cannot start until another finishes. */
export interface DependencyEdge {
 fromTaskId: string;
 toTaskId: string;
}

/** @schema board — a column change, kept so the board's history is replayable. */
export interface Transition {
 id: string;
 taskId: string;
 from: BoardColumn | null;
 to: BoardColumn;
 at: string;
 by: string;
}

/** @schema board — who a task is assigned to, and since when. */
export interface Assignment {
 taskId: string;
 assignee: string;
 at: string;
}

/** @schema board — the run that did the work for a task. */
export interface TaskRunLink {
 taskId: string;
 runId: string;
}

/** @schema board — what proves a task is done: runs, events, artifacts. */
export interface EvidenceLink {
 taskId: string;
 runCount: number;
 eventCount: number;
 artifactIds: string[];
}

/** @schema board — a GitHub issue this task tracks. */
export interface LinkedIssue {
 taskId: string;
 repo: string;
 number: number;
 title: string;
 url: string;
}
