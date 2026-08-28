import type { Task } from "../types/board";

export interface WorkItemProviderRecord {
  pluginId: string;
  label: string;
  host: string;
  project: string;
  resolutionSupported: boolean;
  syncSupported: boolean;
  syncIntervalSeconds: number;
}

export interface ExternalWorkItemIdentity {
  plugin_id: string;
  host: string;
  project: string;
  native_id: string;
}

export interface ExternalWorkItemSnapshot {
  identity: ExternalWorkItemIdentity;
  url: string;
  title: string;
  body: string;
  labels: string[];
  status: string;
}

export interface WorkflowDefinitionRecord {
  id: string;
  name: string;
  version: number;
}

export interface ExternalWorkItemPreview {
  snapshot: ExternalWorkItemSnapshot;
  workflow: {
    taskId: string;
    projectId: string;
    workflowDefId: string | null;
    confirmed: boolean;
  };
}

export type ImportedExternalWorkItemTask = Task;

export type ExternalWorkItemImportResult =
  | { outcome: "imported"; task: ImportedExternalWorkItemTask }
  | { outcome: "existing"; taskId: string };

export interface ExternalWorkItemCompletionStatus {
  taskId: string;
  status: "pending" | "commented" | "resolved" | "failed";
  attempts: number;
  commented: boolean;
  resolved: boolean | null;
  resolutionSupported: boolean;
  error: string | null;
}

export interface ExternalWorkItemSyncState {
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
}

export interface ExternalWorkItemSyncResult {
  taskId: string;
  appliedEvents: number;
  unmappedStatuses: string[];
  echoSuppressedNotes: string[];
  nextCursor: string | null;
  state: ExternalWorkItemSyncState;
}

export interface ExternalWorkItemStatusPushResult {
  taskId: string;
  externalStatus: string;
  state: ExternalWorkItemSyncState;
}

interface PersistedWorkItemProvider {
  pluginId: string;
  host: string;
  project: string;
  comments: boolean;
  resolutionSupported: boolean;
  syncSupported: boolean;
  syncIntervalSeconds: number;
}

function providerRecord(
  provider: PersistedWorkItemProvider,
): WorkItemProviderRecord {
  return {
    pluginId: provider.pluginId,
    label: provider.pluginId,
    host: provider.host,
    project: provider.project,
    resolutionSupported: provider.resolutionSupported,
    syncSupported: provider.syncSupported,
    syncIntervalSeconds: provider.syncIntervalSeconds,
  };
}

export async function loadConfiguredWorkItemProviders(): Promise<
  WorkItemProviderRecord[]
> {
  const { invoke } = await import("@tauri-apps/api/core");
  const providers = await invoke<PersistedWorkItemProvider[]>(
    "external_work_item_providers",
  );
  return providers.map(providerRecord);
}

export async function registerWorkItemProvider(
  config: Omit<
    WorkItemProviderRecord,
    "label" | "resolutionSupported" | "syncSupported"
  >,
): Promise<WorkItemProviderRecord> {
  const { invoke } = await import("@tauri-apps/api/core");
  const provider = await invoke<PersistedWorkItemProvider>(
    "register_external_work_item_provider",
    { request: config },
  );
  return providerRecord(provider);
}

export async function loadExternalWorkItemWorkflows(
  projectId: string,
): Promise<WorkflowDefinitionRecord[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<WorkflowDefinitionRecord[]>("external_work_item_workflows", {
    projectId,
  });
}

export async function loadImportedExternalWorkItemTasks(
  projectId?: string,
): Promise<ImportedExternalWorkItemTask[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ImportedExternalWorkItemTask[]>("external_work_item_tasks", {
    projectId,
  });
}

export async function previewExternalWorkItem(request: {
  pluginId: string;
  host: string;
  project: string;
  nativeId: string;
  projectId: string;
  workflowDefId?: string;
}): Promise<ExternalWorkItemPreview> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemPreview>("preview_external_work_item", {
    request,
  });
}

export async function importExternalWorkItem(
  preview: ExternalWorkItemPreview,
): Promise<ExternalWorkItemImportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemImportResult>("import_external_work_item", {
    request: { preview },
  });
}

export async function completeExternalWorkItem(
  taskId: string,
  evidence: string[] = [],
): Promise<ExternalWorkItemCompletionStatus> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemCompletionStatus>(
    "complete_external_work_item",
    { request: { taskId, evidence } },
  );
}

export async function retryExternalWorkItemCompletion(
  taskId: string,
): Promise<ExternalWorkItemCompletionStatus> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemCompletionStatus>(
    "retry_external_work_item_completion",
    { request: { taskId, evidence: [] } },
  );
}

export async function loadExternalWorkItemCompletionStatus(
  taskId: string,
): Promise<ExternalWorkItemCompletionStatus> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemCompletionStatus>(
    "external_work_item_completion_status",
    { taskId },
  );
}

export async function loadExternalWorkItemSyncState(
  taskId: string,
): Promise<ExternalWorkItemSyncState | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemSyncState | null>(
    "external_work_item_sync_state",
    { taskId },
  );
}

export async function syncExternalWorkItem(
  taskId: string,
): Promise<ExternalWorkItemSyncResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemSyncResult>("sync_external_work_item", {
    taskId,
  });
}

export async function pushExternalWorkItemStatus(
  taskId: string,
): Promise<ExternalWorkItemStatusPushResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<ExternalWorkItemStatusPushResult>(
    "push_external_work_item_status",
    { taskId },
  );
}

export async function pushExternalWorkItemNote(
  taskId: string,
  note: { id: string; body: string; author: string },
): Promise<{ taskId: string; posted: boolean }> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<{ taskId: string; posted: boolean }>(
    "push_external_work_item_note",
    { request: { taskId, ...note } },
  );
}

/** Test fixture for a provider row loaded from board.external_work_item_providers. */
export const GITHUB_WORK_ITEM_PROVIDER_FIXTURE: WorkItemProviderRecord[] = [
  {
    pluginId: "github",
    label: "GitHub",
    host: "github.com",
    project: "forge18/locus",
    resolutionSupported: true,
    syncSupported: true,
    syncIntervalSeconds: 60,
  },
];
