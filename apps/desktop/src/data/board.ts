import { COLUMN_ORDER } from "./demo/fixtures/board";
import type { BoardColumn, DependencyEdge, Task } from "../types/board";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export {
  APPROVAL_NOTE,
  BLOCKED_NOTE,
  COLUMN_LABELS,
  COLUMN_ORDER,
  HEADER_NOTE,
  SECOND_COLUMN_ALTERNATIVE,
  SECOND_COLUMN_LABEL,
} from "./demo/fixtures/board";
export type { BoardColumn, Task } from "../types/board";

export function fetchTasks(projectId?: string): Promise<Envelope<Task[]>> {
  return dataProvider().query<Task>("board_tasks", { projectId });
}

export function createTask(
  projectId: string,
  summary: string,
  workflowDefId: string,
  repoId?: string,
): Promise<Envelope<Task>> {
  return dataProvider().queryOne<Task>("task_create", {
    projectId,
    summary,
    workflowDefId,
    repoId,
  });
}

export function fetchTaskDetail(
  projectId: string,
  taskId: string,
): Promise<Envelope<Task>> {
  return dataProvider().queryOne<Task>("task_detail", { projectId, taskId });
}

export function taskLocator(task: Pick<Task, "projectId" | "id">): string {
  return `locus://${task.projectId}/task/${task.id}`;
}

/** Becomes: invoke("board_tasks") + emit("task_moved") */
export function useTasks(): Task[] {
  return dataProvider().read?.<Task[]>("board_tasks") ?? [];
}

/** Becomes: invoke("board_tasks") — grouped client-side; the columns are fixed. */
export function useTasksByColumn(): Record<BoardColumn, Task[]> {
  const out = Object.fromEntries(
    COLUMN_ORDER.map((c) => [c, [] as Task[]]),
  ) as Record<BoardColumn, Task[]>;
  for (const t of useTasks()) out[t.column].push(t);
  return out;
}

/** Becomes: invoke("board_dependencies") */
export function useDependencies(): DependencyEdge[] {
  return dataProvider().read?.<DependencyEdge[]>("board_dependencies") ?? [];
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
  const evidence =
    dataProvider().read?.<Record<string, { runs: number; events: number }>>(
      "task_evidence",
    );
  return evidence?.[taskId] ?? null;
}
