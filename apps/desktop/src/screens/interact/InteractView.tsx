import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { AgentPane, type AgentPaneSession } from "../../panes/AgentPane";
import {
  commitInteractSession,
  createInteractSession,
  discardInteractSession,
  fetchInteractSessions,
  promoteInteractSession,
  sendInteractPrompt,
  type InteractSessionRow,
} from "../../data/interact";
import { dataProvider } from "../../data/provider";
import { ready, type Envelope } from "../../data/envelope";
import { MergeModal } from "../../shell/MergeModal";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { InlineError } from "../../ui/InlineError";
import { Tag } from "../../ui/Tag";

const OPEN_NOTE =
  "This session has no card, so no approval gate and nothing in your Inbox. This panel is the only account of what it touched.";
const PROMOTED_NOTE =
  "This session was promoted to a card, so its diff now takes the normal gate. What you see here is the record of what it touched before that.";
const DISCARDED_NOTE =
  "This session was discarded. The container and branch are gone; the transcript stays for the record.";

function noteFor(state: InteractSessionRow["state"]): string {
  if (state === "open") return OPEN_NOTE;
  if (state === "promoted") return PROMOTED_NOTE;
  return DISCARDED_NOTE;
}

function ageFor(createdAt: string | null): string {
  if (!createdAt) return "age unknown";
  const elapsed = Math.max(0, Date.now() - Date.parse(createdAt));
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 60) return `${minutes}m`;
  return `${Math.floor(minutes / 60)}h`;
}

function panelStatus(session: InteractSessionRow): AgentPaneSession["status"] {
  if (session.runStatus === "running") return "working";
  if (session.runStatus === "queued" || session.runStatus === "paused")
    return "waiting";
  if (session.runStatus === "completed") return "done";
  return "idle";
}

function panelSession(session: InteractSessionRow): AgentPaneSession {
  return {
    project: session.project,
    task: session.boardTaskId ? `#${session.boardTaskId}` : undefined,
    agent: session.agent,
    model: session.model ?? "unavailable",
    harness: session.harness,
    effort: "unavailable",
    name: session.name,
    cost: session.cost ?? undefined,
    permissionPosture: session.permissionPosture,
    status: panelStatus(session),
  };
}

function localMutation(
  rows: InteractSessionRow[],
  sessionId: string,
  update: (session: InteractSessionRow) => InteractSessionRow,
): InteractSessionRow[] {
  return rows.map((session) =>
    session.id === sessionId ? update(session) : session,
  );
}

export interface InteractViewProps {
  projectId?: string;
}

export function InteractView(props: InteractViewProps = {}) {
  const liveMode = dataProvider().kind === "live";
  const demoRows =
    dataProvider().read?.<InteractSessionRow[]>("interact_sessions_list", {
      projectId: props.projectId,
    }) ?? [];
  const [sessionsEnvelope, setSessionsEnvelope] = createSignal<
    Envelope<InteractSessionRow[]>
  >(liveMode ? { status: "loading" } : ready(demoRows));
  const [selectedId, setSelectedId] = createSignal(demoRows[0]?.id);
  const [collapsed, setCollapsed] = createSignal(false);
  const [research, setResearch] = createSignal(false);
  const [mergeOpen, setMergeOpen] = createSignal(false);
  const [mutationError, setMutationError] = createSignal<string | null>(null);

  const sessions = createMemo(() => {
    const envelope = sessionsEnvelope();
    return envelope.status === "ready" ? envelope.data : [];
  });
  const selected = createMemo(
    () =>
      sessions().find((session) => session.id === selectedId()) ??
      sessions()[0],
  );
  const load = () => {
    if (liveMode)
      void fetchInteractSessions(props.projectId).then(setSessionsEnvelope);
  };

  createEffect(() => {
    const projectId = props.projectId;
    if (liveMode)
      void fetchInteractSessions(projectId).then(setSessionsEnvelope);
  });

  createEffect(() => {
    const rows = sessions();
    if (!rows.some((session) => session.id === selectedId())) {
      setSelectedId(rows[0]?.id);
    }
  });

  const updateDemo = (
    update: (rows: InteractSessionRow[]) => InteractSessionRow[],
  ) => {
    if (liveMode) return;
    const current = sessions();
    setSessionsEnvelope(
      current.length
        ? { status: "ready", data: update(current) }
        : { status: "empty" },
    );
  };

  const discard = (sessionId: string) => {
    setMutationError(null);
    if (!liveMode) {
      updateDemo((rows) =>
        localMutation(rows, sessionId, (session) => ({
          ...session,
          state: "discarded",
          status: "closed",
          runStatus: "cancelled",
        })),
      );
      return;
    }
    void discardInteractSession(props.projectId ?? "", sessionId).then(
      (result) => {
        if (result.status === "failed")
          return setMutationError(result.error.message);
        load();
      },
    );
  };

  const promote = (sessionId: string) => {
    setMutationError(null);
    if (!liveMode) {
      updateDemo((rows) =>
        localMutation(rows, sessionId, (session) => ({
          ...session,
          state: "promoted",
          status: "closed",
          boardTaskId: session.boardTaskId ?? "new",
        })),
      );
      return;
    }
    void promoteInteractSession(props.projectId ?? "", sessionId).then(
      (result) => {
        if (result.status === "failed")
          return setMutationError(result.error.message);
        load();
      },
    );
  };

  const sendPrompt = (session: InteractSessionRow, prompt: string) => {
    if (!liveMode) return;
    void sendInteractPrompt(props.projectId ?? "", session.id, prompt).then(
      (result) => {
        if (result.status === "failed") setMutationError(result.error.message);
      },
    );
  };

  const commit = (session: InteractSessionRow) => {
    setMutationError(null);
    if (!liveMode) {
      setMergeOpen(true);
      return;
    }
    void commitInteractSession(props.projectId ?? "", session.id).then(
      (result) => {
        if (result.status === "failed")
          return setMutationError(result.error.message);
        setMergeOpen(true);
      },
    );
  };

  const create = () => {
    setMutationError(null);
    if (!liveMode) {
      const id = `demo-${Date.now()}`;
      const session: InteractSessionRow = {
        id,
        projectId: props.projectId ?? "tapestry",
        project: props.projectId ?? "tapestry",
        name: "New Interact session",
        agent: "pi@1",
        harness: "pi",
        branch: `interact/${id}`,
        status: "active",
        state: "open",
        boardTaskId: null,
        runId: null,
        runStatus: null,
        model: null,
        permissionPosture: "bypass",
        createdAt: new Date().toISOString(),
        repo: null,
        baseCommit: null,
        changedFiles: [],
        cost: null,
        events: [],
      };
      setSessionsEnvelope({ status: "ready", data: [session, ...sessions()] });
      setSelectedId(id);
      return;
    }
    void createInteractSession(
      props.projectId ?? "",
      "New Interact session",
    ).then((result) => {
      if (result.status === "failed")
        return setMutationError(result.error.message);
      if (result.status === "ready") {
        setSessionsEnvelope((current) => {
          const rows = current.status === "ready" ? current.data : [];
          return { status: "ready", data: [result.data, ...rows] };
        });
        setSelectedId(result.data.id);
      }
    });
  };

  return (
    <div
      class="interact-view"
      data-testid="interact"
      data-collapsed={collapsed() ? "true" : "false"}
    >
      <aside
        class="interact-sessions-rail"
        data-testid="interact-sessions-rail"
      >
        <header>
          <div>
            <h1>Sessions</h1>
            <small>{sessions().length} records</small>
          </div>
          <div>
            <button type="button" data-testid="interact-new" onClick={create}>
              +
            </button>
            <button
              type="button"
              aria-label={collapsed() ? "Expand sessions" : "Collapse sessions"}
              onClick={() => setCollapsed((value) => !value)}
            >
              {collapsed() ? "→" : "◂"}
            </button>
          </div>
        </header>
        <Show
          when={!collapsed()}
          fallback={
            <div class="interact-dot-strip">
              <For each={sessions()}>
                {(session) => (
                  <div class="interact-dot-item">
                    <button
                      type="button"
                      class="interact-session-dot"
                      data-selected={
                        selectedId() === session.id ? "true" : undefined
                      }
                      onClick={() => setSelectedId(session.id)}
                      aria-label={session.name}
                    />
                    <Show when={session.state === "open"}>
                      <button
                        type="button"
                        class="interact-dot-discard"
                        aria-label={`Discard ${session.name}`}
                        onClick={() => discard(session.id)}
                      >
                        ×
                      </button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          }
        >
          <div class="interact-session-list">
            <For each={sessions()}>
              {(session) => (
                <article
                  class="interact-session-card"
                  data-state={session.state}
                  data-selected={
                    selectedId() === session.id ? "true" : undefined
                  }
                  onClick={() => setSelectedId(session.id)}
                >
                  <div>
                    <i class="interact-state-dot" />{" "}
                    <strong>{session.name}</strong>
                    <Show when={session.state === "open"}>
                      <button
                        type="button"
                        aria-label={`Discard ${session.name}`}
                        onClick={(event) => {
                          event.stopPropagation();
                          discard(session.id);
                        }}
                      >
                        ×
                      </button>
                    </Show>
                  </div>
                  <small>
                    {session.harness} · {ageFor(session.createdAt)}
                  </small>
                  <Tag
                    variant={
                      session.state === "discarded" ? "neutral" : "outline"
                    }
                  >
                    {session.state === "open"
                      ? session.changedFiles.length
                        ? `${session.changedFiles.length} changed`
                        : "clean"
                      : session.state === "promoted"
                        ? `→ #${session.boardTaskId}`
                        : "discarded"}
                  </Tag>
                </article>
              )}
            </For>
          </div>
        </Show>
        <footer>
          A session is yours alone — no card, no plan, no gate. Nothing here
          reaches the board unless you promote it.
        </footer>
      </aside>
      <main class="interact-center">
        <Show when={!liveMode}>
          <FixtureNotice
            surface="Interact sessions"
            command='invoke("interact_sessions_list")'
          />
        </Show>
        <Show when={sessionsEnvelope().status === "loading"}>
          <p data-testid="interact-loading">Loading sessions…</p>
        </Show>
        <Show when={sessionsEnvelope().status === "failed"}>
          <InlineError
            cause={
              (
                sessionsEnvelope() as {
                  status: "failed";
                  error: { message: string };
                }
              ).error.message
            }
            next="Reconnect to the Locus store and reopen Interact."
          />
        </Show>
        <Show
          when={sessionsEnvelope().status === "ready" && selected()}
          fallback={
            <Show when={sessionsEnvelope().status === "empty"}>
              <p data-testid="interact-empty">
                A session is a container, a branch and an agent you talk to
                directly. Start one to try something without putting it on the
                board.
              </p>
            </Show>
          }
        >
          {(session) => (
            <>
              <header>
                <div>
                  <Tag variant="outline">Interact</Tag>
                  <h1>{session().name}</h1>
                  <small>{session().harness} · token/cost shown</small>
                </div>
                <button
                  type="button"
                  aria-pressed={research()}
                  onClick={() => setResearch((value) => !value)}
                >
                  {research() ? "Close research" : "Research"}
                </button>
              </header>
              <Show when={mutationError()}>
                {(error) => (
                  <InlineError
                    cause={error()}
                    next="The Interact action was not applied."
                  />
                )}
              </Show>
              <section
                class="interact-agent-panel"
                data-testid="interact-agent-panel"
              >
                <AgentPane
                  runId={session().runId ?? session().id}
                  live={liveMode && Boolean(session().runId)}
                  session={panelSession(session())}
                  events={liveMode ? undefined : session().events}
                  researchOpen={research()}
                  showResearchControl={false}
                  showResearchPane={false}
                  showCost
                  permissionPosture={session().permissionPosture}
                  onSend={(prompt) => sendPrompt(session(), prompt)}
                />
              </section>
              <Show when={session().state === "open"}>
                <footer class="interact-actions">
                  <span class="interact-commit-action">
                    <Button
                      variant="secondary"
                      data-testid="interact-commit"
                      onClick={() => commit(session())}
                    >
                      Commit to branch
                    </Button>
                    <button
                      type="button"
                      data-testid="interact-commit-caret"
                      aria-label="Open merge options"
                      onClick={() => setMergeOpen(true)}
                    >
                      ▾
                    </button>
                  </span>
                  <Button
                    variant="secondary"
                    data-testid="interact-promote"
                    onClick={() => promote(session().id)}
                  >
                    Promote to card
                  </Button>
                  <Button
                    variant="secondary"
                    onClick={() => discard(session().id)}
                  >
                    Discard
                  </Button>
                </footer>
              </Show>
            </>
          )}
        </Show>
      </main>
      <MergeModal
        open={mergeOpen()}
        branch={selected()?.branch}
        repo={selected()?.repo ?? undefined}
        onClose={() => setMergeOpen(false)}
      />
      <aside class="interact-right" data-testid="interact-right-rail">
        <Show
          when={research()}
          fallback={
            <>
              <header>
                <h2>Changed this session</h2>
                <small>
                  repo · {selected()?.repo ?? "unknown"} · base{" "}
                  {selected()?.baseCommit ?? "unknown"}
                </small>
              </header>
              <div class="interact-branch">
                <code>{selected()?.branch ?? "interact/unknown"}</code>
                <span>{selected()?.changedFiles.length ?? 0} files</span>
              </div>
              <For each={selected()?.changedFiles ?? []}>
                {(file) => (
                  <div class="interact-file">
                    <i>{file.marker}</i>
                    <span>{file.path}</span>
                    <small>
                      +{file.additions} −{file.removals}
                    </small>
                  </div>
                )}
              </For>
              <Show when={(selected()?.changedFiles.length ?? 0) === 0}>
                <p>
                  Nothing written yet — this session has only read and run
                  commands.
                </p>
              </Show>
              <Show when={selected()}>
                {(session) => (
                  <p class="interact-state-note">{noteFor(session().state)}</p>
                )}
              </Show>
            </>
          }
        >
          <header>
            <h2>Research</h2>
            <small>Live research is not yet available</small>
          </header>
          <p>Research replaces Changed this session while research is open.</p>
        </Show>
      </aside>
    </div>
  );
}

export default InteractView;
