import { COLUMN_ORDER, DEPENDENCIES, EVIDENCE, TASKS } from "../fixtures/board";
import type { BoardColumn, DependencyEdge, Task } from "../types/board";

export {
  APPROVAL_NOTE,
  BLOCKED_NOTE,
  COLUMN_LABELS,
  COLUMN_ORDER,
  HEADER_NOTE,
  SECOND_COLUMN_ALTERNATIVE,
  SECOND_COLUMN_LABEL,
} from "../fixtures/board";
export type { BoardColumn, Task } from "../types/board";

export function taskLocator(task: Pick<Task, "projectId" | "id">): string {
  return `locus://${task.projectId}/task/${task.id}`;
}

/** Becomes: invoke("board_tasks") + emit("task_moved") */
export function useTasks(): Task[] {
  return TASKS;
}

/** Becomes: invoke("board_tasks") — grouped client-side; the columns are fixed. */
export function useTasksByColumn(): Record<BoardColumn, Task[]> {
  const out = Object.fromEntries(
    COLUMN_ORDER.map((c) => [c, [] as Task[]]),
  ) as Record<BoardColumn, Task[]>;
  for (const t of TASKS) out[t.column].push(t);
  return out;
}

/** Becomes: invoke("board_dependencies") */
export function useDependencies(): DependencyEdge[] {
  return DEPENDENCIES;
}

/** The one manual-creation shape shared by Kanban and List. */
export const MANUAL_TASK_DRAFT = {
  title: "",
  workflowId: null as string | null,
  confirmed: false,
};

/** Becomes: invoke("task_evidence", { taskId }) */
export function useEvidence(
  taskId: string,
): { runs: number; events: number } | null {
  return EVIDENCE[taskId] ?? null;
}
