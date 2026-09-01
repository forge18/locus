import type { AgentEvent } from "../types/event.ts";
import type { AgentPermissionPosture } from "../panes/agent-panel-model.ts";
import type { Envelope } from "./envelope.ts";
import { dataProvider } from "./provider.ts";

export type InteractState = "open" | "promoted" | "discarded";

export interface InteractChangedFile {
  path: string;
  marker: string;
  additions: number;
  removals: number;
}

export interface InteractSessionRow {
  id: string;
  projectId: string;
  project: string;
  name: string;
  agent: string;
  harness: string;
  branch: string;
  status: string;
  state: InteractState;
  boardTaskId: string | null;
  runId: string | null;
  runStatus: string | null;
  model: string | null;
  permissionPosture: AgentPermissionPosture;
  createdAt: string | null;
  repo: string | null;
  baseCommit: string | null;
  changedFiles: InteractChangedFile[];
  cost: string | null;
  events?: AgentEvent[];
}

export interface InteractMutationResult {
  session: InteractSessionRow;
  branch?: string;
}

export function fetchInteractSessions(
  projectId?: string,
): Promise<Envelope<InteractSessionRow[]>> {
  return dataProvider().query<InteractSessionRow>("interact_sessions_list", {
    projectId,
  });
}

export function createInteractSession(
  projectId: string,
  name: string,
  model?: string,
  repoId?: string,
): Promise<Envelope<InteractSessionRow>> {
  return dataProvider().queryOne<InteractSessionRow>(
    "interact_session_create",
    { projectId, name, model, repoId },
  );
}

export function promoteInteractSession(
  projectId: string,
  sessionId: string,
  taskId?: string,
): Promise<Envelope<InteractSessionRow>> {
  return dataProvider().queryOne<InteractSessionRow>(
    "interact_session_promote",
    { projectId, sessionId, taskId },
  );
}

export function discardInteractSession(
  projectId: string,
  sessionId: string,
): Promise<Envelope<InteractMutationResult>> {
  return dataProvider().queryOne<InteractMutationResult>(
    "interact_session_discard",
    { projectId, sessionId },
  );
}

export function commitInteractSession(
  projectId: string,
  sessionId: string,
): Promise<Envelope<InteractMutationResult>> {
  return dataProvider().queryOne<InteractMutationResult>(
    "interact_session_commit",
    { projectId, sessionId },
  );
}

export function sendInteractPrompt(
  projectId: string,
  sessionId: string,
  prompt: string,
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>("interact_session_prompt", {
    projectId,
    sessionId,
    prompt,
  });
}
