import { For, Show, createSignal } from "solid-js";
import { Button } from "../../ui/Button";
import { Segmented } from "../../ui/Segmented";
import "./manage.css";

type ManageViewKind = "kanban" | "list" | "graph" | "timeline";
const COLUMNS = [
  "Ready",
  "In Progress",
  "Testing",
  "Reviewing",
  "Pending Approval",
  "Done",
] as const;
const TASKS = [
  {
    id: "t-1184",
    title: "Add normalized event query",
    column: "Testing",
    state: "verify: cargo test",
    blocked: false,
  },
  {
    id: "t-1185",
    title: "Reconcile provider aliases",
    column: "Reviewing",
    state: "Gate: reviewer agent",
    blocked: true,
  },
  {
    id: "t-1186",
    title: "Write QA adapter",
    column: "In Progress",
    state: "builder · 41.2k tokens",
    blocked: false,
  },
];

export function ManageView() {
  const [view, setView] = createSignal<ManageViewKind>("kanban");
  const [hideDone, setHideDone] = createSignal(false);
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
          <Button variant="secondary">Import task</Button>
          <Button variant="primary">Add task</Button>
        </div>
      </header>
      <Show when={view() === "kanban"}>
        <main class="manage-kanban">
          <header>
            <h1>Manage</h1>
            <span>n cards · 3 in flight per person</span>
            <label>
              <input
                type="checkbox"
                checked={hideDone()}
                onChange={(event) => setHideDone(event.currentTarget.checked)}
              />{" "}
              Hide Done
            </label>
          </header>
          <div class="manage-columns">
            <For each={COLUMNS}>
              {(column) => (
                <section data-column={column}>
                  <h2>
                    {column}{" "}
                    <small>
                      {TASKS.filter((task) => task.column === column).length}
                    </small>
                  </h2>
                  <For
                    each={TASKS.filter(
                      (task) =>
                        task.column === column &&
                        (!hideDone() || column !== "Done"),
                    )}
                  >
                    {(task) => (
                      <article
                        class="manage-task-card"
                        data-testid={`manage-task-${task.id}`}
                      >
                        <strong>{task.title}</strong>
                        <small>{task.state}</small>
                        <Show when={task.blocked}>
                          <Tag>blocked: approval</Tag>
                        </Show>
                      </article>
                    )}
                  </For>
                </section>
              )}
            </For>
          </div>
          <footer class="manage-dwell">
            The two slowest columns are the two that need a human. Agents are
            not the constraint here — the median card spends thirty-eight
            minutes being built and seventeen hours waiting to be looked at.
          </footer>
        </main>
      </Show>
      <Show when={view() === "list"}>
        <main class="manage-list">
          <header>
            <h1>Sessions</h1>
            <span>Sorted by needs-attention, then activity.</span>
          </header>
          <section class="manage-session-detail">
            <h2>Live</h2>
            <p>
              Iteration 3/8 · 2 tool errors against baseline · 41.2k tokens ·
              last file write 2m ago
            </p>
            <div class="manage-guardrail">
              <strong>
                Guardrail — kill &amp; reassign after 3 stuck iterations
              </strong>
              <p>
                Handoff drafted: 3 done, 2 remaining, 4 attempted, 1 open. The
                successor reads the payload, never this transcript.
              </p>
              <Button variant="primary">Hand off to reviewer@2</Button>
              <Button variant="secondary">Let it run</Button>
            </div>
          </section>
        </main>
      </Show>
      <Show when={view() === "graph"}>
        <main class="manage-graph">
          <h1>Dependency graph</h1>
          <p>Left to right is dependency depth, not time.</p>
          <div class="manage-graph-edges">
            <span class="edge-grey">t-1184 ───▶ t-1185</span>
            <span class="edge-amber">t-1186 ───▶ t-1185</span>
          </div>
          <aside>
            <h2>Unblocks most</h2>
            <p>
              Two of the four cards holding up the most work are waiting on a
              human, not an agent — the same story the dwell chart tells.
            </p>
          </aside>
        </main>
      </Show>
      <Show when={view() === "timeline"}>
        <main class="manage-timeline">
          <h1>Timeline</h1>
          <p>grouped by workflow · last 7 days</p>
          <div class="manage-axis">Mon · Tue · Wed · Thu · Fri · Sat · Sun</div>
          <For each={TASKS}>
            {(task) => (
              <div class="manage-swimlane">
                <strong>{task.title}</strong>
                <span class="timeline-segment ready" />
                <span class="timeline-segment working" />
                <span class="timeline-segment blocked">
                  {task.column} · wall-clock
                </span>
              </div>
            )}
          </For>
          <footer>
            Bar length is wall-clock, not agent time. The widest bars are almost
            entirely amber and slate.
          </footer>
        </main>
      </Show>
    </div>
  );
}
function Tag(props: { children: string }) {
  return <span class="manage-tag">{props.children}</span>;
}
export default ManageView;
