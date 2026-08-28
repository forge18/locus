import { For, Show, createSignal } from "solid-js";
import { Button } from "../../ui/Button";
import { Tag } from "../../ui/Tag";
import { MergeModal } from "../../shell/MergeModal";
import { AgentPane, type AgentPaneSession } from "../../panes/AgentPane";
import type { AgentEvent } from "../../types/event";
import "./interact.css";

type InteractState = "open" | "promoted" | "discarded";
interface InteractSession {
  id: string;
  name: string;
  harness: string;
  age: string;
  state: InteractState;
  changed: number;
  task?: string;
}

const SESSIONS: InteractSession[] = [
  {
    id: "r-9f21",
    name: "Try the notification path",
    harness: "claude",
    age: "4m",
    state: "open",
    changed: 2,
  },
  {
    id: "r-9c02",
    name: "Review parser behavior",
    harness: "codex",
    age: "18m",
    state: "promoted",
    changed: 4,
    task: "1184",
  },
  {
    id: "r-8a11",
    name: "Discarded experiment",
    harness: "pi",
    age: "2h",
    state: "discarded",
    changed: 1,
  },
];

const noteFor = (state: InteractState) =>
  state === "open"
    ? "This session has no card, so no approval gate and nothing in your Inbox. This panel is the only account of what it touched."
    : state === "promoted"
      ? "This session was promoted to a card, so its diff now takes the normal gate. What you see here is the record of what it touched before that."
      : "This session was discarded. The container and branch are gone; the transcript stays for the record.";

const panelSession = (session: InteractSession): AgentPaneSession => ({
  project: "tapestry",
  task: session.task ?? session.name,
  agent: `${session.harness}@1`,
  model: "session-model",
  harness: session.harness,
  effort: "high",
  name: session.name,
  context: { used: 12_400, total: 200_000 },
  cost: "$0.42",
  permissionPosture: "bypass",
  status: session.state === "open" ? "working" : session.state === "promoted" ? "done" : "idle",
});

const panelEvents = (runId: string): AgentEvent[] => [
  {
    id: `${runId}-user`,
    runId,
    seq: 0,
    ts: "now",
    verb: "user",
    text: "Try this without putting it on the board.",
    raw: { source: "interact" },
  },
  {
    id: `${runId}-assistant`,
    runId,
    seq: 1,
    ts: "now",
    verb: "assistant",
    text: "I am reading the repository and will leave a compact change summary.",
    raw: { source: "interact" },
  },
];

export function InteractView() {
  const [sessions, setSessions] = createSignal(SESSIONS);
  const [selectedId, setSelectedId] = createSignal(SESSIONS[0].id);
  const [collapsed, setCollapsed] = createSignal(false);
  const [research, setResearch] = createSignal(false);
  const [mergeOpen, setMergeOpen] = createSignal(false);
  const selected = () =>
    sessions().find((session) => session.id === selectedId()) ?? sessions()[0];
  const discard = (id: string) =>
    setSessions((current) =>
      current.map((session) =>
        session.id === id && session.state === "open"
          ? { ...session, state: "discarded" }
          : session,
      ),
    );

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
            <small>{sessions().length} open records</small>
          </div>
          <button
            type="button"
            aria-label="Collapse sessions"
            onClick={() => setCollapsed((value) => !value)}
          >
            ◂
          </button>
        </header>
        <Show
          when={!collapsed()}
          fallback={
            <div class="interact-dot-strip">
              <For each={sessions()}>
                {(session) => (
                  <button
                    type="button"
                    data-selected={
                      selectedId() === session.id ? "true" : undefined
                    }
                    onClick={() => setSelectedId(session.id)}
                    aria-label={session.name}
                  />
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
                  </div>
                  <small>
                    {session.harness} · {session.age}
                  </small>
                  <Tag
                    variant={
                      session.state === "discarded" ? "neutral" : "outline"
                    }
                  >
                    {session.state === "open"
                      ? session.changed
                        ? `${session.changed} changed`
                        : "clean"
                      : session.state === "promoted"
                        ? `→ #${session.task}`
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
        <header>
          <div>
            <Tag variant="outline">Interact</Tag>
            <h1>{selected().name}</h1>
            <small>{selected().harness} · token/cost shown</small>
          </div>
          <button
            type="button"
            aria-pressed={research()}
            onClick={() => setResearch((value) => !value)}
          >
            {research() ? "Close research" : "Research"}
          </button>
        </header>
        <section
          class="interact-agent-panel"
          data-testid="interact-agent-panel"
        >
          <AgentPane
            runId={selected().id}
            live={false}
            session={panelSession(selected())}
            events={panelEvents(selected().id)}
            researchOpen={research()}
            showResearchControl={false}
            showResearchPane={false}
          />
        </section>
        <Show when={selected().state === "open"}>
          <footer class="interact-actions">
            <Button
              variant="secondary"
              data-testid="interact-commit"
              onClick={() => setMergeOpen(true)}
            >
              Commit to branch
            </Button>
            <Button variant="secondary" onClick={() => discard(selected().id)}>
              Discard
            </Button>
          </footer>
        </Show>
      </main>
      <MergeModal
        open={mergeOpen()}
        branch={`interact/${selected().id}`}
        onClose={() => setMergeOpen(false)}
      />
      <aside class="interact-right" data-testid="interact-right-rail">
        <Show
          when={research()}
          fallback={
            <>
              <header>
                <h2>Changed this session</h2>
                <small>repo · tapestry · base 6da8e0b</small>
              </header>
              <div class="interact-branch">
                <code>interact/{selected().id}</code>
                <span>{selected().changed} files</span>
              </div>
              <For
                each={
                  selected().changed
                    ? [
                        "crates/locus-core/src/notify.rs",
                        "apps/desktop/src/screens/interact/InteractView.tsx",
                      ].slice(0, selected().changed)
                    : []
                }
              >
                {(path) => (
                  <div class="interact-file">
                    <i>M</i>
                    <span>{path}</span>
                    <small>+12 −3</small>
                  </div>
                )}
              </For>
              <Show when={!selected().changed}>
                <p>
                  Nothing written yet — this session has only read and run
                  commands.
                </p>
              </Show>
              <p class="interact-state-note">{noteFor(selected().state)}</p>
            </>
          }
        >
          <header>
            <h2>Research</h2>
            <small>2 of 4 came from the plan</small>
          </header>
          <p>Findings replace Changed this session while research is open.</p>
          <ul>
            <li>seed · source citation</li>
            <li>this run · claim → source</li>
          </ul>
        </Show>
      </aside>
    </div>
  );
}
export default InteractView;
