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

export interface PlanningWorkspaceSpec {
  id: string;
  workspaceId: string;
  repoId: string;
  name: string;
  state: Record<string, unknown>;
  stale: boolean;
  updatedAt: string;
}

export interface PlanningWorkspaceSession {
  workspaceId: string;
  specId: string | null;
  sessionId: string;
  linkedAt: string;
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

export function listPlanningWorkspaceSpecs(
  projectId: string,
  workspaceId: string,
): Promise<Envelope<PlanningWorkspaceSpec[]>> {
  return dataProvider().query<PlanningWorkspaceSpec>(
    "planning_workspace_specs_list",
    { projectId, workspaceId },
  );
}

export function savePlanningWorkspaceSpec(
  projectId: string,
  workspaceId: string,
  repoId: string,
  name: string,
  state: Record<string, unknown>,
  stale: boolean,
  specId?: string,
): Promise<Envelope<PlanningWorkspaceSpec>> {
  return dataProvider().queryOne<PlanningWorkspaceSpec>(
    "planning_workspace_spec_save",
    { projectId, workspaceId, specId, repoId, name, state, stale },
  );
}

export function recordPlanningWorkspaceDecision(
  projectId: string,
  workspaceId: string,
  affectedSpecIds: string[],
  decision: Record<string, unknown>,
): Promise<Envelope<{ updated: boolean }>> {
  return dataProvider().queryOne<{ updated: boolean }>(
    "planning_workspace_decision_record",
    { projectId, workspaceId, affectedSpecIds, decision },
  );
}

export function listPlanningWorkspaceSessions(
  projectId: string,
  workspaceId: string,
): Promise<Envelope<PlanningWorkspaceSession[]>> {
  return dataProvider().query<PlanningWorkspaceSession>(
    "planning_workspace_sessions_list",
    { projectId, workspaceId },
  );
}

export function linkPlanningWorkspaceSession(
  projectId: string,
  workspaceId: string,
  sessionId: string,
  specId?: string,
): Promise<Envelope<boolean>> {
  return dataProvider().queryOne<boolean>("planning_workspace_session_link", {
    projectId,
    workspaceId,
    sessionId,
    specId,
  });
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
