import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Segmented } from "../../ui/Segmented";
import { Tag } from "../../ui/Tag";
import {
  COLUMN_LABELS,
  COLUMN_ORDER,
  MANUAL_TASK_DRAFT,
  taskLocator,
  useDependencies,
  useTasks,
} from "../../data/board";
import {
  completeExternalWorkItem,
  importExternalWorkItem,
  loadConfiguredWorkItemProviders,
  loadImportedExternalWorkItemTasks,
  loadExternalWorkItemCompletionStatus,
  loadExternalWorkItemSyncState,
  loadExternalWorkItemWorkflows,
  previewExternalWorkItem,
  pushExternalWorkItemNote,
  pushExternalWorkItemStatus,
  retryExternalWorkItemCompletion,
  syncExternalWorkItem,
  type ExternalWorkItemCompletionStatus,
  type ExternalWorkItemPreview,
  type ExternalWorkItemSyncResult,
  type ExternalWorkItemSyncState,
  type WorkflowDefinitionRecord,
  type WorkItemProviderRecord,
} from "../../data/work-items";
import type { BoardColumn, Task } from "../../types/board";
import "./manage.css";

type ManageViewKind = "kanban" | "list" | "graph" | "timeline";

type DraftSource = "kanban" | "list";

function taskStatus(task: Task): string {
  if (task.status === "stuck")
    return `stuck · ${task.stuckIterations}/${task.maxIterations}`;
  if (task.status === "blocked") return "blocked · dependency";
  return task.assignee
    ? `${task.assignee} · ${task.tokens ?? "ready"}`
    : "unassigned";
}

function safeExternalUrl(task: Task): string | undefined {
  if (!task.externalLink) return undefined;
  try {
    const url = new URL(task.externalLink);
    const expectedHost = task.externalHost ?? "github.com";
    const defaultPort = url.port === "" || url.port === "443";
    return url.protocol === "https:" &&
      url.hostname.toLowerCase() === expectedHost.toLowerCase() &&
      defaultPort
      ? url.href
      : undefined;
  } catch {
    return undefined;
  }
}

function emptySyncState(): ExternalWorkItemSyncState {
  return {
    pullCursor: null,
    lastPushedStatus: null,
    noteWatermark: null,
    lastLocalStatusAt: null,
    lastExternalStatusAt: null,
    lastSyncError: null,
    lastSyncedAt: null,
    unmappedExternalStatus: null,
    lastConflictWinner: null,
    lastConflictReason: null,
  };
}

function TaskDraft(props: { source: DraftSource; onClose: () => void }) {
  return (
    <section
      class="manage-task-draft"
      data-testid={`automate-create-task-${props.source}`}
      data-draft-contract="manual-task"
    >
      <header>
        <h2>New task</h2>
        <span>manual task draft · workflow confirmation required</span>
      </header>
      <label>
        Summary
        <input
          placeholder="What should be done?"
          value={MANUAL_TASK_DRAFT.title}
        />
      </label>
      <p>
        Workflow: <strong>select and confirm before start</strong>
      </p>
      <div>
        <Button variant="primary">Create draft</Button>
        <Button variant="ghost" onClick={props.onClose}>
          Cancel
        </Button>
      </div>
    </section>
  );
}

function TaskImport(props: {
  source: DraftSource;
  onClose: () => void;
  providers: WorkItemProviderRecord[];
  workflows: WorkflowDefinitionRecord[];
  projectId?: string;
  onExistingTask?: (taskId: string) => void;
  onImported?: (task: Task) => void;
}) {
  const [provider, setProvider] = createSignal(props.providers[0]);
  const [issueNumber, setIssueNumber] = createSignal("");
  const [workflowDefId, setWorkflowDefId] = createSignal(
    props.workflows[0]?.id ?? "",
  );
  const [preview, setPreview] = createSignal<ExternalWorkItemPreview>();
  const [error, setError] = createSignal<string>();
  const [busy, setBusy] = createSignal(false);
  const [existingTaskId, setExistingTaskId] = createSignal<string>();
  createEffect(() => {
    if (!provider() && props.providers[0]) setProvider(props.providers[0]);
    if (!workflowDefId() && props.workflows[0]) {
      setWorkflowDefId(props.workflows[0].id);
    }
  });

  const loadPreview = async (selected: WorkItemProviderRecord) => {
    if (!props.projectId) {
      setError("A local project is required before importing an issue.");
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      setPreview(
        await previewExternalWorkItem({
          pluginId: selected.pluginId,
          host: selected.host,
          project: selected.project,
          nativeId: issueNumber(),
          projectId: props.projectId,
          workflowDefId: workflowDefId() || undefined,
        }),
      );
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  };

  const confirmImport = async (selected: WorkItemProviderRecord) => {
    const current = preview();
    if (!current) {
      await loadPreview(selected);
      return;
    }
    if (!workflowDefId().trim()) {
      setError("A workflow definition is required before import.");
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const result = await importExternalWorkItem({
        ...current,
        workflow: {
          ...current.workflow,
          workflowDefId: workflowDefId().trim(),
          confirmed: true,
        },
      });
      if (result.outcome === "existing") {
        setExistingTaskId(result.taskId);
      } else {
        props.onImported?.(result.task);
        props.onClose();
      }
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section
      class="manage-task-draft"
      data-testid={`automate-import-${props.source}`}
      data-import-contract="external-work-item"
    >
      <header>
        <h2>Import work item</h2>
        <span>configured provider · preview before local task creation</span>
      </header>
      <Show
        when={provider()}
        fallback={
          <p data-testid="automate-import-no-provider">
            No work-item plugin is configured for this project.
          </p>
        }
      >
        {(selected) => (
          <>
            <div
              class="manage-import-providers"
              data-testid="automate-import-providers"
            >
              <For each={props.providers}>
                {(candidate) => (
                  <button
                    type="button"
                    aria-pressed={
                      `${selected().pluginId}:${selected().host}/${selected().project}` ===
                      `${candidate.pluginId}:${candidate.host}/${candidate.project}`
                    }
                    data-plugin-id={candidate.pluginId}
                    onClick={() => {
                      setProvider(candidate);
                      setPreview(undefined);
                      setExistingTaskId(undefined);
                    }}
                  >
                    {candidate.label}
                  </button>
                )}
              </For>
            </div>
            <article
              class="manage-import-preview"
              data-testid="automate-import-preview"
              data-provider={selected().pluginId}
            >
              <label>
                Issue number
                <input
                  aria-label="GitHub issue number"
                  value={issueNumber()}
                  onInput={(event) => {
                    setIssueNumber(event.currentTarget.value);
                    setPreview(undefined);
                    setExistingTaskId(undefined);
                    setError(undefined);
                  }}
                />
              </label>
              <Show
                when={preview()}
                fallback={
                  <strong>
                    {selected().label} · Issue #{issueNumber() || "…"} ·
                    Normalize event delivery
                  </strong>
                }
              >
                {(loaded) => (
                  <strong>
                    {selected().label} · Issue #
                    {loaded().snapshot.identity.native_id} ·{" "}
                    {loaded().snapshot.title}
                  </strong>
                )}
              </Show>
              <p>
                {selected().host}/{selected().project} ·{" "}
                {selected().resolutionSupported
                  ? "comment and resolution supported"
                  : "comment only; resolution unsupported"}
              </p>
              <p>
                {selected().syncSupported
                  ? "Statuses and notes synchronize through the provider; completion delivery remains separate."
                  : "Snapshot imported once; provider edits do not synchronize into Locus."}
              </p>
            </article>
            <label>
              Workflow
              <select
                aria-label="Import workflow"
                data-testid="automate-import-workflow"
                value={workflowDefId()}
                onChange={(event) =>
                  setWorkflowDefId(event.currentTarget.value)
                }
              >
                <option value="">Select a workflow definition</option>
                <For each={props.workflows}>
                  {(workflow) => (
                    <option value={workflow.id}>
                      {workflow.name} · v{workflow.version}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <Show when={!props.workflows.length}>
              <p data-testid="automate-import-no-workflow">
                No workflow definition is available for this project.
              </p>
            </Show>
            <p data-testid="automate-import-one-way">
              {selected().syncSupported
                ? "Status and note synchronization is enabled for this provider."
                : "No source write before local Done."}
            </p>
            <Show when={error()}>
              {(message) => <p role="alert">{message()}</p>}
            </Show>
            <Show when={existingTaskId()}>
              {(taskId) => (
                <p data-testid="automate-import-existing">
                  Already imported as task {taskId()}.{" "}
                  <button
                    type="button"
                    onClick={() => {
                      props.onExistingTask?.(taskId());
                      props.onClose();
                    }}
                  >
                    Open existing task
                  </button>
                </p>
              )}
            </Show>
            <div>
              <Button
                variant="primary"
                disabled={busy()}
                onClick={() => void confirmImport(selected())}
              >
                {preview() ? "Confirm import" : "Load preview"}
              </Button>
              <Button variant="ghost" onClick={props.onClose}>
                Cancel
              </Button>
            </div>
          </>
        )}
      </Show>
    </section>
  );
}

function TaskDetail(props: { task: Task }) {
  const task = props.task;
  const externalUrl = safeExternalUrl(task);
  const [completion, setCompletion] =
    createSignal<ExternalWorkItemCompletionStatus>();
  const [completionError, setCompletionError] = createSignal<string>();
  const [syncState, setSyncState] = createSignal<ExternalWorkItemSyncState>(
    task.syncState ?? emptySyncState(),
  );
  const [syncResult, setSyncResult] =
    createSignal<ExternalWorkItemSyncResult>();
  const [syncError, setSyncError] = createSignal<string>();
  const [syncBusy, setSyncBusy] = createSignal(false);
  const applyCompletion = (result: ExternalWorkItemCompletionStatus) => {
    setCompletion(result);
    setCompletionError(result.error ?? undefined);
  };
  const refreshCompletion = () => {
    if (!externalUrl) return;
    void loadExternalWorkItemCompletionStatus(task.id)
      .then(applyCompletion)
      .catch((caught) => setCompletionError(String(caught)));
  };
  const applySync = (result: ExternalWorkItemSyncResult) => {
    setSyncResult(result);
    setSyncState(result.state);
    setSyncError(result.state.lastSyncError ?? undefined);
  };
  const refreshSync = () => {
    if (!externalUrl || !task.syncSupported) return;
    void loadExternalWorkItemSyncState(task.id)
      .then((state) => {
        if (state) setSyncState(state);
      })
      .catch((caught) => setSyncError(String(caught)));
  };
  const syncNow = () => {
    if (!externalUrl || !task.syncSupported) return;
    setSyncBusy(true);
    setSyncError(undefined);
    void syncExternalWorkItem(task.id)
      .then(applySync)
      .catch((caught) => setSyncError(String(caught)))
      .finally(() => setSyncBusy(false));
  };
  const pushStatus = () => {
    if (!externalUrl || !task.syncSupported) return;
    setSyncBusy(true);
    setSyncError(undefined);
    void pushExternalWorkItemStatus(task.id)
      .then((result) => setSyncState(result.state))
      .catch((caught) => setSyncError(String(caught)))
      .finally(() => setSyncBusy(false));
  };
  const [noteBody, setNoteBody] = createSignal("");
  const pushNote = () => {
    const body = noteBody().trim();
    if (!externalUrl || !task.syncSupported || !body) return;
    setSyncBusy(true);
    setSyncError(undefined);
    void pushExternalWorkItemNote(task.id, {
      id: `note-${Date.now()}`,
      body,
      author: "human",
    })
      .then(() => setNoteBody(""))
      .catch((caught) => setSyncError(String(caught)))
      .finally(() => setSyncBusy(false));
  };
  onMount(() => {
    refreshCompletion();
    refreshSync();
  });
  const syncStatus = () => {
    if (syncError() || syncState().lastSyncError) return "failed";
    if (syncState().unmappedExternalStatus) return "unmapped";
    if (syncState().lastSyncedAt) return "synced";
    return "not synced";
  };
  const completionStatus = () =>
    completion()?.status ?? task.completionStatus ?? "pending";
  const completionAttempts = () =>
    completion()?.attempts ?? task.completionAttempts ?? 0;
  const resolutionSupported = () =>
    completion()?.resolutionSupported ?? task.resolutionSupported;
  return (
    <section
      class="manage-task-detail"
      data-testid="automate-task-detail"
      data-task-locator={taskLocator(task)}
    >
      <header>
        <div>
          <span class="manage-detail-kicker">Task detail</span>
          <h2>{task.title}</h2>
        </div>
        <code>{taskLocator(task)}</code>
      </header>
      <dl>
        <div>
          <dt>Workflow</dt>
          <dd>{task.workflowId ?? "select and confirm before start"}</dd>
        </div>
        <div>
          <dt>Root session</dt>
          <dd>{task.rootSessionId ?? "not started"}</dd>
        </div>
        <div>
          <dt>Workers</dt>
          <dd>{task.childRunIds?.length ?? 0} child runs</dd>
        </div>
        <div>
          <dt>Evidence</dt>
          <dd>{task.evidenceIds?.length ?? 0} linked items</dd>
        </div>
      </dl>
      <Show when={task.comments?.length}>
        <section
          class="manage-task-comments"
          data-testid="automate-task-comments"
        >
          <strong>Notes</strong>
          <For each={task.comments ?? []}>
            {(comment) => (
              <article data-comment-origin={comment.origin}>
                <span>{comment.author}</span>
                <p>{comment.body}</p>
              </article>
            )}
          </For>
        </section>
      </Show>
      <Show when={task.childRunIds?.length}>
        <div class="manage-detail-runs" data-testid="automate-task-runs">
          <strong>Run tree</strong>
          <For each={task.childRunIds}>{(runId) => <code>{runId}</code>}</For>
        </div>
      </Show>
      <div class="manage-detail-actions" data-testid="automate-task-controls">
        <Show when={externalUrl && task.column !== "done"}>
          <Button
            variant="primary"
            onClick={() => {
              setCompletionError(undefined);
              void completeExternalWorkItem(task.id, task.evidenceIds ?? [])
                .then(applyCompletion)
                .catch((caught) => setCompletionError(String(caught)));
            }}
          >
            Mark Done and deliver
          </Button>
        </Show>
        <Button variant="secondary">Pause</Button>
        <Button variant="secondary">Cancel</Button>
        <Button variant="secondary">Hand off</Button>
        <Button variant="ghost">Needs attention</Button>
      </div>
      <div
        class="manage-import-completion"
        data-testid="automate-import-completion-status"
        data-completion-status={completionStatus()}
      >
        Completion delivery: {completionStatus()} · {completionAttempts()}{" "}
        attempts ·{" "}
        {resolutionSupported() === false
          ? "resolution unsupported"
          : "one idempotent comment after local Done"}
        <Show when={completionError()}>
          {(message) => <span role="alert"> · {message()}</span>}
        </Show>
        <Show
          when={
            externalUrl &&
            task.column === "done" &&
            (completionStatus() === "failed" ||
              completionStatus() === "pending")
          }
        >
          <Button
            variant="ghost"
            onClick={() => {
              setCompletionError(undefined);
              void retryExternalWorkItemCompletion(task.id)
                .then(applyCompletion)
                .catch((caught) => setCompletionError(String(caught)));
            }}
          >
            Retry delivery
          </Button>
        </Show>
      </div>
      <Show when={externalUrl && task.syncSupported}>
        <section
          class="manage-import-sync"
          data-testid="automate-sync-status"
          data-sync-status={syncStatus()}
        >
          <div class="manage-import-sync-heading">
            <strong>Synchronization: {syncStatus()}</strong>
            <Show when={syncState().lastSyncedAt}>
              <small>Last synced {syncState().lastSyncedAt}</small>
            </Show>
          </div>
          <Show when={syncState().unmappedExternalStatus}>
            {(status) => (
              <p data-testid="automate-sync-unmapped">
                Unmapped external status: {status()}
              </p>
            )}
          </Show>
          <Show when={syncState().lastConflictWinner}>
            {(winner) => (
              <p data-testid="automate-sync-conflict">
                Last conflict: {winner()} won. {syncState().lastConflictReason}
              </p>
            )}
          </Show>
          <Show when={syncResult()}>
            {(result) => (
              <p data-testid="automate-sync-result">
                Applied {result().appliedEvents} sync events; suppressed{" "}
                {result().echoSuppressedNotes.length} echoed notes.
              </p>
            )}
          </Show>
          <Show when={syncError()}>
            {(message) => <p role="alert">{message()}</p>}
          </Show>
          <div class="manage-import-sync-actions">
            <Button variant="secondary" disabled={syncBusy()} onClick={syncNow}>
              {syncBusy() ? "Syncing…" : "Sync now"}
            </Button>
            <Button variant="ghost" disabled={syncBusy()} onClick={pushStatus}>
              Push current status
            </Button>
          </div>
          <form
            class="manage-import-sync-note"
            onSubmit={(event) => {
              event.preventDefault();
              pushNote();
            }}
          >
            <input
              aria-label="External note"
              placeholder="Add a note to the external item"
              value={noteBody()}
              onInput={(event) => setNoteBody(event.currentTarget.value)}
            />
            <Button type="submit" variant="ghost" disabled={syncBusy()}>
              Post note
            </Button>
          </form>
        </section>
      </Show>
      <Show when={externalUrl}>
        <a href={externalUrl}>External work item</a>
      </Show>
    </section>
  );
}

export interface ManageViewProps {
  workItemProviders?: WorkItemProviderRecord[];
  workflowDefinitions?: WorkflowDefinitionRecord[];
  projectId?: string;
}

export function ManageView(props: ManageViewProps = {}) {
  const [view, setView] = createSignal<ManageViewKind>("kanban");
  const [loadedWorkItemProviders, setLoadedWorkItemProviders] = createSignal<
    WorkItemProviderRecord[]
  >([]);
  const [loadedWorkflows, setLoadedWorkflows] = createSignal<
    WorkflowDefinitionRecord[]
  >([]);
  const [importedTasks, setImportedTasks] = createSignal<Task[]>([]);
  onMount(() => {
    if (props.workItemProviders === undefined) {
      void loadConfiguredWorkItemProviders()
        .then(setLoadedWorkItemProviders)
        .catch(() => setLoadedWorkItemProviders([]));
    }
  });
  createEffect(() => {
    const projectId = props.projectId;
    setImportedTasks([]);
    setSelectedTaskId(null);
    if (props.workflowDefinitions === undefined) setLoadedWorkflows([]);
    if (!projectId) {
      if (props.workflowDefinitions === undefined) setLoadedWorkflows([]);
      return;
    }
    let active = true;
    void loadImportedExternalWorkItemTasks(projectId)
      .then((tasks) => {
        if (active) setImportedTasks(tasks);
      })
      .catch(() => {
        if (active) setImportedTasks([]);
      });
    if (props.workflowDefinitions === undefined) {
      void loadExternalWorkItemWorkflows(projectId)
        .then((workflows) => {
          if (active) setLoadedWorkflows(workflows);
        })
        .catch(() => {
          if (active) setLoadedWorkflows([]);
        });
    }
    onCleanup(() => {
      active = false;
    });
  });
  const [hideDone, setHideDone] = createSignal(false);
  const [selectedTaskId, setSelectedTaskId] = createSignal<string | null>(null);
  const [draftSource, setDraftSource] = createSignal<DraftSource>();
  const [importSource, setImportSource] = createSignal<DraftSource>();
  const tasks = createMemo(() => [...useTasks(), ...importedTasks()]);
  const tasksByColumn = createMemo(() => {
    const grouped = Object.fromEntries(
      COLUMN_ORDER.map((column) => [column, [] as Task[]]),
    ) as Record<BoardColumn, Task[]>;
    for (const task of tasks()) grouped[task.column].push(task);
    return grouped;
  });
  const selectedTask = () => {
    const taskId = selectedTaskId();
    return taskId ? tasks().find((task) => task.id === taskId) : tasks()[0];
  };
  const openTask = (task: Task) => setSelectedTaskId(task.id);
  const openDraft = () => {
    const current = view();
    if (current === "kanban" || current === "list") setDraftSource(current);
  };
  const openImport = () => {
    const current = view();
    if (current === "kanban" || current === "list") setImportSource(current);
  };

  return (
    <div class="manage-view" data-testid="manage" data-view={view()}>
      <FixtureNotice surface="Manage" command='invoke("board_tasks")' />
      <header class="manage-toolbar">
        <Segmented
          options={[
            { value: "kanban", label: "Kanban" },
            { value: "list", label: "List" },
            { value: "graph", label: "Graph" },
            { value: "timeline", label: "Timeline" },
          ]}
          value={view()}
          onChange={(value) => setView(value as ManageViewKind)}
          label="Manage view"
        />
        <div>
          <Button variant="secondary" onClick={openImport}>
            Import task
          </Button>
          <Button variant="primary" onClick={openDraft}>
            Add task
          </Button>
        </div>
      </header>
      <Show when={draftSource()}>
        {(source) => (
          <TaskDraft
            source={source()}
            onClose={() => setDraftSource(undefined)}
          />
        )}
      </Show>
      <Show when={importSource()}>
        {(source) => (
          <TaskImport
            source={source()}
            onClose={() => setImportSource(undefined)}
            providers={props.workItemProviders ?? loadedWorkItemProviders()}
            workflows={props.workflowDefinitions ?? loadedWorkflows()}
            projectId={props.projectId}
            onExistingTask={(taskId) => setSelectedTaskId(taskId)}
            onImported={(task) => {
              setImportedTasks((current) => [...current, task]);
              setSelectedTaskId(task.id);
            }}
          />
        )}
      </Show>

      <Show when={view() === "kanban"}>
        <main class="manage-kanban">
          <header>
            <h1>Tasks</h1>
            <span>{tasks().length} tasks · 3 in flight per person</span>
            <label>
              <input
                type="checkbox"
                checked={hideDone()}
                onChange={(event) => setHideDone(event.currentTarget.checked)}
              />{" "}
              Hide Done
            </label>
          </header>
          <div class="manage-columns" data-testid="automate-kanban-tasks">
            <For each={COLUMN_ORDER}>
              {(column: BoardColumn) => (
                <section
                  data-column={column}
                  data-column-label={COLUMN_LABELS[column]}
                >
                  <h2>
                    {COLUMN_LABELS[column]}{" "}
                    <small>{tasksByColumn()[column].length}</small>
                  </h2>
                  <For
                    each={
                      hideDone() && column === "done"
                        ? []
                        : tasksByColumn()[column]
                    }
                  >
                    {(task) => (
                      <button
                        type="button"
                        class="manage-task-card"
                        data-testid={`manage-task-${task.id}`}
                        data-task-locator={taskLocator(task)}
                        onClick={() => openTask(task)}
                      >
                        <strong>{task.title}</strong>
                        <small>{taskStatus(task)}</small>
                        <small>{task.verifyCommand}</small>
                        <Show when={task.ciStatus}>
                          <small data-testid={`kanban-ci-${task.id}`}>
                            CI · {task.ciStatus}
                          </small>
                        </Show>
                        <Show when={task.status !== "ok"}>
                          <Tag variant="neutral">{task.status}</Tag>
                        </Show>
                      </button>
                    )}
                  </For>
                </section>
              )}
            </For>
          </div>
          <footer class="manage-dwell">
            Blocked is a status, not a column. Dependencies clear automatically
            without moving a card.
          </footer>
        </main>
      </Show>

      <Show when={view() === "list"}>
        <main class="manage-list">
          <header>
            <h1>Tasks</h1>
            <span>Same project-scoped task query as Kanban.</span>
          </header>
          <section class="manage-task-list" data-testid="automate-list-tasks">
            <For each={tasks()}>
              {(task) => (
                <button
                  type="button"
                  class="manage-task-list-row"
                  data-testid={`manage-list-task-${task.id}`}
                  data-task-locator={taskLocator(task)}
                  onClick={() => openTask(task)}
                >
                  <strong>{task.title}</strong>
                  <span>{COLUMN_LABELS[task.column]}</span>
                  <span>{taskStatus(task)}</span>
                  <code>{task.verifyCommand}</code>
                </button>
              )}
            </For>
          </section>
        </main>
      </Show>

      <Show when={view() === "graph"}>
        <main class="manage-graph">
          <h1>Dependency graph</h1>
          <p>Left to right is dependency depth, not time.</p>
          <div class="manage-graph-edges">
            <For each={useDependencies()}>
              {(edge) => (
                <span class="edge-grey">
                  {edge.fromTaskId} ───▶ {edge.toTaskId}
                </span>
              )}
            </For>
          </div>
          <aside>
            <h2>Unblocks most</h2>
            <p>Workflow-generated dependencies are the only edge source.</p>
          </aside>
        </main>
      </Show>

      <Show when={view() === "timeline"}>
        <main class="manage-timeline">
          <h1>Timeline</h1>
          <p>grouped by task · last 7 days</p>
          <div class="manage-axis">Mon · Tue · Wed · Thu · Fri · Sat · Sun</div>
          <For each={tasks()}>
            {(task) => (
              <div
                class="manage-swimlane"
                data-task-locator={taskLocator(task)}
              >
                <strong>{task.title}</strong>
                <span class="timeline-segment ready" />
                <span class="timeline-segment working" />
                <span class="timeline-segment blocked">
                  {COLUMN_LABELS[task.column]} · wall-clock
                </span>
              </div>
            )}
          </For>
          <footer>Bar length is wall-clock, not agent time.</footer>
        </main>
      </Show>

      <Show
        keyed
        when={(view() === "kanban" || view() === "list") && selectedTask()}
      >
        {(task) => <TaskDetail task={task} />}
      </Show>
    </div>
  );
}

export default ManageView;
