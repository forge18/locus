import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import { fetchProjects } from "../../data/core";
import {
  approvePlanningWorkspace,
  createPlanningWorkspace,
  deletePlanningWorkspace,
  listPlanningWorkspaceRevisions,
  listPlanningWorkspaceSessions,
  listPlanningWorkspaceSpecs,
  listPlanningWorkspaces,
  savePlanningWorkspaceCheckpoint,
  type PlanningWorkspace,
  type PlanningWorkspaceLifecycle,
  type PlanningWorkspaceRevision,
  type PlanningWorkspaceScope,
  type PlanningWorkspaceSession,
  type PlanningWorkspaceSpec,
} from "../../data/planning-workspace";
import type { Envelope } from "../../data/envelope";
import type { NavStore } from "../../nav";
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";

const SCOPES: PlanningWorkspaceScope[] = ["amendment", "feature", "project"];
const ROOM_TABS = ["brief", "shape", "specs", "tasks", "coverage", "activity"] as const;
type RoomTab = (typeof ROOM_TABS)[number];
type Props = { nav?: NavStore };
type CheckpointState = Record<string, unknown>;

function stringsIn(state: CheckpointState, key: string): string[] {
  const value = state[key];
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === "string")
    : [];
}

function objectsIn(state: CheckpointState, key: string): CheckpointState[] {
  const value = state[key];
  return Array.isArray(value)
    ? value.filter(
        (entry): entry is CheckpointState =>
          Boolean(entry) && typeof entry === "object" && !Array.isArray(entry),
      )
    : [];
}

function textIn(state: CheckpointState, key: string): string {
  return typeof state[key] === "string" ? String(state[key]) : "—";
}

function errorFrom<T>(envelope: Envelope<T>): string | null {
  return envelope.status === "failed" ? envelope.error.message : null;
}

function workspaceLabel(workspace: PlanningWorkspace, projectName: string) {
  return `${workspace.scope} · ${projectName}`;
}

export function PlanningWorkspaceView(props: Props) {
  const [workspaceEnvelope, setWorkspaceEnvelope] = createSignal<
    Envelope<PlanningWorkspace[]>
  >({ status: "loading" });
  const [projectEnvelope, setProjectEnvelope] = createSignal<
    Envelope<{ id: string; name: string }[]>
  >({ status: "loading" });
  const [revisionEnvelope, setRevisionEnvelope] = createSignal<
    Envelope<PlanningWorkspaceRevision[]>
  >({ status: "empty" });
  const [specEnvelope, setSpecEnvelope] = createSignal<
    Envelope<PlanningWorkspaceSpec[]>
  >({ status: "empty" });
  const [sessionEnvelope, setSessionEnvelope] = createSignal<
    Envelope<PlanningWorkspaceSession[]>
  >({ status: "empty" });
  const [selectedId, setSelectedId] = createSignal("");
  const [brief, setBrief] = createSignal("");
  const [scope, setScope] = createSignal<PlanningWorkspaceScope>("feature");
  const [projectId, setProjectId] = createSignal(props.nav?.params().project ?? "");
  const [stateText, setStateText] = createSignal("{}");
  const [tab, setTab] = createSignal<RoomTab>("brief");
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);

  const workspaces = createMemo(() => {
    const envelope = workspaceEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const projects = createMemo(() => {
    const envelope = projectEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const selected = createMemo(
    () => workspaces().find((workspace) => workspace.id === selectedId()) ?? null,
  );
  const revisions = createMemo(() => {
    const envelope = revisionEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const specs = createMemo(() => {
    const envelope = specEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const sessions = createMemo(() => {
    const envelope = sessionEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const currentRevision = createMemo(
    () => revisions().find((revision) => revision.revision === selected()?.currentRevision) ?? null,
  );
  const projectName = (id: string) => projects().find((project) => project.id === id)?.name ?? id;
  const currentState = createMemo<CheckpointState>(() => {
    const revision = currentRevision();
    return revision?.state ?? {};
  });
  const stateSpecs = createMemo(() => objectsIn(currentState(), "specs"));
  const stateTasks = createMemo(() => objectsIn(currentState(), "tasks"));
  const stateRequirements = createMemo(() =>
    stateSpecs().flatMap((spec) => objectsIn(spec, "requirements")),
  );

  async function load() {
    const [workspaceRows, projectRows] = await Promise.all([
      listPlanningWorkspaces(),
      fetchProjects(),
    ]);
    setWorkspaceEnvelope(workspaceRows);
    setProjectEnvelope(projectRows);
    if (workspaceRows.status === "ready") {
      const next = workspaceRows.data.find((workspace) => workspace.id === selectedId()) ?? workspaceRows.data[0];
      setSelectedId(next?.id ?? "");
    }
  }

  async function loadRevisions(id: string) {
    setRevisionEnvelope({ status: "loading" });
    setSpecEnvelope({ status: "loading" });
    setSessionEnvelope({ status: "loading" });
    const workspace = workspaces().find((row) => row.id === id);
    if (!workspace) {
      setRevisionEnvelope({ status: "empty" });
      setSpecEnvelope({ status: "empty" });
      setSessionEnvelope({ status: "empty" });
      return;
    }
    const [revisionRows, specRows, sessionRows] = await Promise.all([
      listPlanningWorkspaceRevisions(workspace.projectId, id),
      listPlanningWorkspaceSpecs(workspace.projectId, id),
      listPlanningWorkspaceSessions(workspace.projectId, id),
    ]);
    setRevisionEnvelope(revisionRows);
    setSpecEnvelope(specRows);
    setSessionEnvelope(sessionRows);
    if (revisionRows.status === "ready") {
      const revision = revisionRows.data.find((row) => row.revision === workspace.currentRevision) ?? revisionRows.data[revisionRows.data.length - 1];
      if (revision) setStateText(JSON.stringify(revision.state, null, 2));
    }
  }

  createEffect(() => {
    const id = selectedId();
    if (id) void loadRevisions(id);
  });

  onMount(() => void load());

  async function createWorkspace() {
    const selectedProject = projectId();
    const nextBrief = brief().trim();
    if (!selectedProject || !nextBrief) return;
    setBusy(true);
    setMessage(null);
    const envelope = await createPlanningWorkspace(selectedProject, scope(), nextBrief);
    if (envelope.status === "ready") {
      setBrief("");
      await load();
      setSelectedId(envelope.data.id);
    } else if (envelope.status === "failed") {
      setMessage(envelope.error.message);
    }
    setBusy(false);
  }

  async function saveCheckpoint() {
    const workspace = selected();
    if (!workspace) return;
    let state: Record<string, unknown>;
    try {
      const parsed: unknown = JSON.parse(stateText());
      if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) throw new Error("state must be a JSON object");
      state = parsed as Record<string, unknown>;
    } catch (cause) {
      setMessage(cause instanceof Error ? cause.message : String(cause));
      return;
    }
    const lifecycle: Exclude<PlanningWorkspaceLifecycle, "approved"> =
      workspace.lifecycle === "ready_for_approval" ? "ready_for_approval" : "in_progress";
    setBusy(true);
    setMessage(null);
    const envelope = await savePlanningWorkspaceCheckpoint(
      workspace.projectId,
      workspace.id,
      workspace.currentRevision,
      lifecycle,
      state,
    );
    if (envelope.status === "ready") {
      await load();
      await loadRevisions(workspace.id);
    } else if (envelope.status === "failed") setMessage(envelope.error.message);
    setBusy(false);
  }

  async function approve() {
    const workspace = selected();
    if (!workspace || workspace.lifecycle !== "ready_for_approval") return;
    setBusy(true);
    setMessage(null);
    const envelope = await approvePlanningWorkspace(
      workspace.projectId,
      workspace.id,
      workspace.currentRevision,
    );
    if (envelope.status === "ready") {
      setMessage(`Approved ${envelope.data.taskIds.length} board task(s).`);
      await load();
    } else if (envelope.status === "failed") setMessage(envelope.error.message);
    setBusy(false);
  }

  async function remove() {
    const workspace = selected();
    if (!workspace) return;
    setBusy(true);
    setMessage(null);
    const envelope = await deletePlanningWorkspace(workspace.projectId, workspace.id);
    if (envelope.status === "failed") setMessage(envelope.error.message);
    else {
      setSelectedId("");
      await load();
    }
    setBusy(false);
  }

  const loadError = createMemo(() => errorFrom(workspaceEnvelope()) ?? errorFrom(projectEnvelope()));

  return (
    <div class="plan-workspace planning-workspace-live" data-testid="planning-workspace">
      <header class="plan-workspace-head">
        <span class="plan-workspace-toggle">
          Planning workspaces <span class="mono">{workspaces().length}</span>
        </span>
        <span class="planning-workspace-note">
          Durable checkpoints; board tasks appear only after approval.
        </span>
      </header>
      <Show when={loadError()}>
        <InlineError
          cause={loadError()!}
          next="The planning workspace data could not be loaded from the live store."
        />
      </Show>
      <div class="planning-workspace-grid">
        <aside class="planning-workspace-list">
          <section class="planning-workspace-create">
            <h2>New workspace</h2>
            <label>
              Project
              <select
                value={projectId()}
                onChange={(event) => setProjectId(event.currentTarget.value)}
              >
                <option value="">Choose a project</option>
                <For each={projects()}>
                  {(project) => <option value={project.id}>{project.name}</option>}
                </For>
              </select>
            </label>
            <label>
              Scope
              <select
                value={scope()}
                onChange={(event) =>
                  setScope(event.currentTarget.value as PlanningWorkspaceScope)
                }
              >
                <For each={SCOPES}>
                  {(entry) => <option value={entry}>{entry}</option>}
                </For>
              </select>
            </label>
            <label>
              Brief
              <textarea
                value={brief()}
                onInput={(event) => setBrief(event.currentTarget.value)}
                placeholder="What should this workspace plan?"
              />
            </label>
            <Button
              variant="primary"
              block
              disabled={busy() || !projectId() || !brief().trim()}
              onClick={() => void createWorkspace()}
            >
              Create workspace
            </Button>
          </section>
          <Show when={workspaceEnvelope().status === "empty"}>
            <p class="planning-workspace-empty">No planning workspaces yet.</p>
          </Show>
          <For each={workspaces()}>
            {(workspace) => (
              <button
                type="button"
                class="planning-workspace-card"
                aria-selected={workspace.id === selectedId()}
                onClick={() => setSelectedId(workspace.id)}
              >
                <strong>{workspace.scope}</strong>
                <span>
                  {workspaceLabel(workspace, projectName(workspace.projectId))}
                </span>
                <small>
                  {workspace.lifecycle} · revision {workspace.currentRevision}
                </small>
              </button>
            )}
          </For>
        </aside>
        <main class="planning-workspace-detail">
          <Show when={selected()} fallback={<UnavailablePanel />}>
            {(workspace) => (
              <>
                <header class="planning-workspace-detail-head">
                  <div>
                    <span class="plan-stage-label">
                      {workspace().lifecycle}
                    </span>
                    <h1>{workspace().scope} workspace</h1>
                    <p>
                      {projectName(workspace().projectId)} · revision{" "}
                      {workspace().currentRevision}
                    </p>
                  </div>
                  <div class="planning-workspace-actions">
                    <Button
                      disabled={
                        busy() || workspace().lifecycle === "approved"
                      }
                      onClick={() => void saveCheckpoint()}
                    >
                      Save checkpoint
                    </Button>
                    <Button
                      variant="primary"
                      disabled={
                        busy() || workspace().lifecycle !== "ready_for_approval"
                      }
                      onClick={() => void approve()}
                    >
                      Approve tasks
                    </Button>
                    <Button
                      disabled={
                        busy() ||
                        !["draft", "in_progress"].includes(
                          workspace().lifecycle,
                        )
                      }
                      onClick={() => void remove()}
                    >
                      Delete draft
                    </Button>
                  </div>
                </header>
                <nav
                  class="planning-room-tabs"
                  role="tablist"
                  aria-label="Planning room"
                >
                  <For each={ROOM_TABS}>
                    {(entry) => (
                      <button
                        type="button"
                        role="tab"
                        aria-selected={tab() === entry}
                        onClick={() => setTab(entry)}
                      >
                        {entry}
                      </button>
                    )}
                  </For>
                </nav>
                <Switch>
                  <Match when={tab() === "brief"}>
                    <section class="planning-room-panel">
                      <h2>Brief</h2>
                      <p class="planning-room-lead">
                        {textIn(currentState(), "brief")}
                      </p>
                      <RoomList title="Use cases" items={stringsIn(currentState(), "useCases")} />
                      <RoomList title="Success criteria" items={stringsIn(currentState(), "successCriteria")} />
                    </section>
                  </Match>
                  <Match when={tab() === "shape"}>
                    <section class="planning-room-panel">
                      <h2>Shape</h2>
                      <dl class="planning-room-facts">
                        <dt>Scope</dt><dd>{workspace().scope}</dd>
                        <dt>Depth</dt><dd>{textIn(currentState(), "depth")}</dd>
                      </dl>
                      <RoomList title="Risks" items={stringsIn(currentState(), "risks")} />
                      <RoomList title="Open questions" items={stringsIn(currentState(), "openQuestions")} />
                    </section>
                  </Match>
                  <Match when={tab() === "specs"}>
                    <section class="planning-room-panel">
                      <h2>Specs</h2>
                      <Show
                        when={specs().length > 0}
                        fallback={<RoomList title="Checkpoint specs" items={stateSpecs().map((spec) => textIn(spec, "name"))} />}
                      >
                        <For each={specs()}>
                          {(spec) => (
                            <article class="planning-room-card">
                              <strong>{spec.name}</strong>
                              <span>repo {spec.repoId}</span>
                              <small>{spec.stale ? "stale" : "current"} · updated {spec.updatedAt}</small>
                            </article>
                          )}
                        </For>
                      </Show>
                    </section>
                  </Match>
                  <Match when={tab() === "tasks"}>
                    <section class="planning-room-panel">
                      <h2>Tasks</h2>
                      <Show when={stateTasks().length > 0} fallback={<p>No tasks are in this checkpoint.</p>}>
                        <For each={stateTasks()}>
                          {(task) => (
                            <article class="planning-room-card">
                              <strong>{textIn(task, "summary")}</strong>
                              <span>{textIn(task, "verify")}</span>
                              <small>after {stringsIn(task, "after").join(", ") || "nothing"}</small>
                            </article>
                          )}
                        </For>
                      </Show>
                    </section>
                  </Match>
                  <Match when={tab() === "coverage"}>
                    <section class="planning-room-panel">
                      <h2>Coverage</h2>
                      <p class="planning-room-lead">Every requirement must map to a task or an explicit non-task outcome before approval.</p>
                      <For each={stateRequirements()}>
                        {(requirement) => (
                          <article class="planning-room-coverage">
                            <strong>{textIn(requirement, "id")}</strong>
                            <span>{textIn(requirement, "body")}</span>
                            <small>{stringsIn(requirement, "taskIds").join(", ") || (requirement.nonTaskOutcome === true ? "non-task outcome" : "uncovered")}</small>
                          </article>
                        )}
                      </For>
                    </section>
                  </Match>
                  <Match when={tab() === "activity"}>
                    <section class="planning-room-panel">
                      <h2>Activity</h2>
                      <For each={revisions()}>
                        {(revision) => (
                          <article class="planning-room-card">
                            <strong>Revision {revision.revision}</strong>
                            <span>{revision.frozenAt ? "frozen" : "editable"}{revision.approvedAt ? " · approved" : ""}</span>
                            <small>{revision.id}</small>
                          </article>
                        )}
                      </For>
                      <Show when={sessions().length > 0}>
                        <RoomList title="Planning sessions" items={sessions().map((session) => session.sessionId)} />
                      </Show>
                      <Show when={currentRevision()}>
                        <label class="planning-room-raw">
                          Raw checkpoint state
                          <textarea value={stateText()} onInput={(event) => setStateText(event.currentTarget.value)} aria-label="Workspace checkpoint state" />
                        </label>
                      </Show>
                    </section>
                  </Match>
                </Switch>
                <Show when={message()}>
                  <p class="planning-workspace-message">{message()}</p>
                </Show>
                <Show when={revisionEnvelope().status === "loading"}>
                  <p>Loading checkpoint…</p>
                </Show>
              </>
            )}
          </Show>
        </main>
      </div>
    </div>
  );
}

function RoomList(props: { title: string; items: string[] }) {
  return (
    <section class="planning-room-list">
      <h3>{props.title}</h3>
      <Show when={props.items.length > 0} fallback={<p>None recorded.</p>}>
        <ul>
          <For each={props.items}>{(item) => <li>{item}</li>}</For>
        </ul>
      </Show>
    </section>
  );
}

function UnavailablePanel() {
  return (
    <section class="plan-stage-panel" data-testid="planning-workspace-empty-detail">
      <h2>Select or create a workspace</h2>
      <p>Planning state is persisted per project and remains resumable until you approve or delete it.</p>
    </section>
  );
}
