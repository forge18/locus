import { For, Show, createSignal } from "solid-js";
import { Button } from "../../ui/Button";
import { Segmented } from "../../ui/Segmented";
import { Tag } from "../../ui/Tag";
import {
  COLUMN_LABELS,
  COLUMN_ORDER,
  MANUAL_TASK_DRAFT,
  taskLocator,
  useDependencies,
  useTasks,
  useTasksByColumn,
} from "../../data/board";
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

function TaskImport(props: { source: DraftSource; onClose: () => void }) {
  const providers = [
    "GitHub",
    "GitLab",
    "Codeberg",
    "Bitbucket Cloud",
    "Bitbucket Data Center",
    "Jira Cloud",
    "Jira Data Center",
  ];
  const [provider, setProvider] = createSignal(providers[0]);

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
      <div
        class="manage-import-providers"
        data-testid="automate-import-providers"
      >
        <For each={providers}>
          {(candidate) => (
            <button
              type="button"
              aria-pressed={provider() === candidate}
              onClick={() => setProvider(candidate)}
            >
              {candidate}
            </button>
          )}
        </For>
      </div>
      <article
        class="manage-import-preview"
        data-testid="automate-import-preview"
        data-provider={provider()}
      >
        <strong>{provider()} · Issue #42 · Normalize event delivery</strong>
        <p>
          Snapshot imported once; provider edits do not synchronize into Locus.
        </p>
      </article>
      <label>
        Workflow
        <select
          aria-label="Import workflow"
          data-testid="automate-import-workflow"
        >
          <option>Project default · confirm before start</option>
        </select>
      </label>
      <p data-testid="automate-import-one-way">
        No source write before local Done.
      </p>
      <div>
        <Button variant="primary" onClick={props.onClose}>
          Preview and confirm
        </Button>
        <Button variant="ghost" onClick={props.onClose}>
          Cancel
        </Button>
      </div>
    </section>
  );
}

function TaskDetail(props: { task: Task }) {
  const task = props.task;
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
      <Show when={task.childRunIds?.length}>
        <div class="manage-detail-runs" data-testid="automate-task-runs">
          <strong>Run tree</strong>
          <For each={task.childRunIds}>{(runId) => <code>{runId}</code>}</For>
        </div>
      </Show>
      <div class="manage-detail-actions" data-testid="automate-task-controls">
        <Button variant="secondary">Pause</Button>
        <Button variant="secondary">Cancel</Button>
        <Button variant="secondary">Hand off</Button>
        <Button variant="ghost">Needs attention</Button>
      </div>
      <div
        class="manage-import-completion"
        data-testid="automate-import-completion-status"
        data-completion-status={
          task.completionStatus ?? (task.externalLink ? "resolved" : "pending")
        }
      >
        Completion delivery:{" "}
        {task.completionStatus ?? (task.externalLink ? "resolved" : "pending")}{" "}
        · {task.completionAttempts ?? 0} attempts ·{" "}
        {task.resolutionSupported === false
          ? "resolution unsupported"
          : "one idempotent comment after local Done"}
      </div>
      <Show when={task.externalLink}>
        <a href={task.externalLink!}>External work item</a>
      </Show>
    </section>
  );
}

export function ManageView() {
  const [view, setView] = createSignal<ManageViewKind>("kanban");
  const [hideDone, setHideDone] = createSignal(false);
  const [selectedTaskId, setSelectedTaskId] = createSignal<string | null>(null);
  const [draftSource, setDraftSource] = createSignal<DraftSource>();
  const [importSource, setImportSource] = createSignal<DraftSource>();
  const tasks = useTasks();
  const tasksByColumn = useTasksByColumn();
  const selectedTask = () =>
    tasks.find((task) => task.id === selectedTaskId()) ?? tasks[0];
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
          />
        )}
      </Show>

      <Show when={view() === "kanban"}>
        <main class="manage-kanban">
          <header>
            <h1>Tasks</h1>
            <span>{tasks.length} tasks · 3 in flight per person</span>
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
                    <small>{tasksByColumn[column].length}</small>
                  </h2>
                  <For
                    each={
                      hideDone() && column === "done"
                        ? []
                        : tasksByColumn[column]
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
            <For each={tasks}>
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
          <For each={tasks}>
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

      <Show when={(view() === "kanban" || view() === "list") && selectedTask()}>
        {(task) => <TaskDetail task={task()} />}
      </Show>
    </div>
  );
}

export default ManageView;
