import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export type PlanningWorkspaceScope = "amendment" | "feature" | "project";
export type PlanningWorkspaceLifecycle =
  | "draft"
  | "in_progress"
  | "ready_for_approval"
  | "approved";

export interface PlanningWorkspace {
  id: string;
  projectId: string;
  scope: PlanningWorkspaceScope;
  lifecycle: PlanningWorkspaceLifecycle;
  currentRevision: number;
  updatedAt: string;
}

export interface PlanningWorkspaceRevision {
  id: string;
  workspaceId: string;
  revision: number;
  state: Record<string, unknown>;
  frozenAt: string | null;
  approvedAt: string | null;
}

export function listPlanningWorkspaces(
  projectId?: string,
): Promise<Envelope<PlanningWorkspace[]>> {
  return dataProvider().query<PlanningWorkspace>("planning_workspaces_list", {
    projectId,
  });
}

export function createPlanningWorkspace(
  projectId: string,
  scope: PlanningWorkspaceScope,
  brief: string,
): Promise<Envelope<PlanningWorkspace>> {
  return dataProvider().queryOne<PlanningWorkspace>("planning_workspace_create", {
    projectId,
    scope,
    brief,
  });
}

export function listPlanningWorkspaceRevisions(
  projectId: string,
  workspaceId: string,
): Promise<Envelope<PlanningWorkspaceRevision[]>> {
  return dataProvider().query<PlanningWorkspaceRevision>(
    "planning_workspace_revisions_list",
    { projectId, workspaceId },
  );
}

export function savePlanningWorkspaceCheckpoint(
  projectId: string,
  workspaceId: string,
  expectedRevision: number,
  lifecycle: Exclude<PlanningWorkspaceLifecycle, "approved">,
  state: Record<string, unknown>,
): Promise<Envelope<PlanningWorkspace>> {
  return dataProvider().queryOne<PlanningWorkspace>(
    "planning_workspace_checkpoint_save",
    { projectId, workspaceId, expectedRevision, lifecycle, state },
  );
}

export interface PlanningWorkspaceApproval {
  workspaceId: string;
  revision: number;
  taskIds: string[];
}

export function approvePlanningWorkspace(
  projectId: string,
  workspaceId: string,
  expectedRevision: number,
): Promise<Envelope<PlanningWorkspaceApproval>> {
  return dataProvider().queryOne<PlanningWorkspaceApproval>(
    "planning_workspace_approve",
    { projectId, workspaceId, expectedRevision },
  );
}

export function deletePlanningWorkspace(
  projectId: string,
  workspaceId: string,
): Promise<Envelope<boolean>> {
  return dataProvider().queryOne<boolean>("planning_workspace_delete", {
    projectId,
    workspaceId,
  });
}
