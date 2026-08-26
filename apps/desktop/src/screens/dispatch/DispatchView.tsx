import { For, Show, createSignal } from "solid-js";
import {
  DISPATCH_PROJECTS,
  SCHEDULE_EXECUTIONS,
  SCHEDULES,
  STOP_ALL_AGENT_COUNT,
  STOP_ALL_RESTORE_MINUTES,
  NEVER_AUTORUN_EXCLUSIONS,
  VERIFY_VOCABULARY,
  autorunMasterState,
  useDispatchRuns,
  type AutorunState,
  type PermissionPosture,
} from "../../data/dispatch";
import { Button } from "../../ui/Button";
import { Segmented } from "../../ui/Segmented";

import "./dispatch.css";

export type DispatchTab = "autorun" | "schedules" | "runs";

export interface DispatchViewProps {
  /** The fixture route decides which Dispatch sub-surface is rendered. */
  tab: DispatchTab;
}

const DISPATCH_TABS = [
  { value: "autorun", label: "Autorun" },
  { value: "schedules", label: "Schedules" },
  { value: "runs", label: "Runs" },
];

const RUN_ROWS = useDispatchRuns();
const RUN_FIXTURE_ROWS = [
  ...RUN_ROWS.slice(0, 11),
  RUN_ROWS.find((row) => row.tokens === null)!,
];

function AutorunSwitch(props: {
  state: AutorunState;
  onToggle: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      class="dispatch-switch"
      data-on={props.state === "on" ? "true" : undefined}
      aria-pressed={props.state === "on"}
      aria-label={`Autorun ${props.state}`}
      disabled={props.disabled}
      onClick={props.onToggle}
    >
      <span />
    </button>
  );
}

function DispatchTabs(props: { active: DispatchTab }) {
  return (
    <Segmented
      options={DISPATCH_TABS}
      value={props.active}
      onChange={() => undefined}
      label="Dispatch"
    />
  );
}

function AutorunView() {
  const [projects, setProjects] = createSignal(DISPATCH_PROJECTS);
  const [stopOpen, setStopOpen] = createSignal(false);
  const [stopped, setStopped] = createSignal(false);
  const [handoff, setHandoff] = createSignal(true);
  const master = () => autorunMasterState(projects());

  const toggleProject = (id: string) => {
    setProjects((current) =>
      current.map((project) =>
        project.id === id && (project.state === "on" || project.state === "off")
          ? { ...project, state: project.state === "on" ? "off" : "on" }
          : project,
      ),
    );
  };

  return (
    <div class="dispatch-view" data-testid="dispatch-autorun">
      <header class="dispatch-header">
        <DispatchTabs active="autorun" />
        <span class="dispatch-header-note">
          5 projects · 3 running · 1 review slot free
        </span>
        <Button variant="secondary">Pause everything</Button>
        <Button variant="secondary" onClick={() => setStopOpen(true)}>
          Stop all
        </Button>
      </header>

      <Show when={stopped()}>
        <div class="dispatch-stopped" data-testid="dispatch-stopped">
          <strong>Everything is stopped.</strong>
          <span>
            8 agents killed 2s ago — 8 handoffs written, nothing lost. Autorun
            is off in all five projects and 3 schedules will skip until you arm
            them.
          </span>
          <Button variant="secondary" data-testid="stop-all-restore">
            Restore previous state
          </Button>
        </div>
      </Show>

      <div class="dispatch-scroll">
        <section class="dispatch-section">
          <div
            class="dispatch-master"
            data-testid="autorun-master"
            data-state={master().label.toLowerCase().replace(" ", "-")}
          >
            <strong>{master().label}</strong>
            <span>
              {master().eligible} eligible projects · {master().on} on
            </span>
          </div>
          <h1>Autorun is on or off, per project</h1>
          <p>
            On means agents in that project pick up their own work and run it
            without you starting anything. Off means every run begins with you,
            or with a schedule you wrote.
          </p>
          <div class="autorun-projects" data-testid="autorun-projects">
            <For each={projects()}>
              {(project) => {
                const unavailable = () =>
                  project.state === "archived" || project.state === "suspended";
                return (
                  <article
                    class="autorun-project"
                    data-testid={`autorun-project-${project.id}`}
                    data-project={project.id}
                    data-state={project.state}
                  >
                    <AutorunSwitch
                      state={project.state}
                      disabled={unavailable()}
                      onToggle={() => toggleProject(project.id)}
                    />
                    <div class="autorun-project-name">
                      <strong>#{project.name}</strong>
                      <span>{project.repos}</span>
                    </div>
                    <span class="autorun-project-state">{project.state}</span>
                    <p>{project.detail}</p>
                    <span class="autorun-project-activity">
                      {project.activity}
                    </span>
                  </article>
                );
              }}
            </For>
          </div>
        </section>

        <div class="dispatch-grid">
          <section class="dispatch-card">
            <h2>What holds it back when it is on</h2>
            <strong class="dispatch-metric" data-testid="autorun-review-debt">
              3 <span>of 4 review slots in use · 1 free</span>
            </strong>
            <div class="dispatch-slots" aria-label="3 of 4 review slots">
              <i />
              <i />
              <i />
              <i />
            </div>
            <p>
              A slot is one change you have not reviewed yet, not one agent.
              Autorun drains at the rate you absorb, or it is just a way of
              generating a backlog faster.
            </p>
            <dl class="dispatch-policy-values">
              <div>
                <dt>Review debt</dt>
                <dd>3 landed, unread</dd>
              </div>
              <div>
                <dt>Pauses at</dt>
                <dd>4 changes</dd>
              </div>
              <div>
                <dt>Inbox budget</dt>
                <dd>6 / hour</dd>
              </div>
              <div>
                <dt>Change ceiling</dt>
                <dd>400 lines · 12 files</dd>
              </div>
            </dl>
          </section>
          <section class="dispatch-card">
            <h2>Never autoruns</h2>
            <ul>
              <For each={NEVER_AUTORUN_EXCLUSIONS}>
                {(exclusion) => (
                  <li data-testid={`never-autorun-${exclusion.id}`}>
                    <strong>{exclusion.label}</strong>
                    <span> — {exclusion.reason}</span>
                  </li>
                )}
              </For>
            </ul>
          </section>
        </div>
      </div>

      <Show when={stopOpen()}>
        <div class="dispatch-dialog-backdrop">
          <section
            class="dispatch-dialog"
            role="dialog"
            aria-modal="true"
            aria-label="Stop everything"
            data-testid="stop-all-dialog"
          >
            <h2>Stop everything</h2>
            <p>
              Kills every active agent and turns dispatch off across the
              install. Nothing restarts on its own — not autorun, not a
              schedule, not a babysitter.
            </p>
            <ul>
              <li>
                {STOP_ALL_AGENT_COUNT} running agents — killed at the next
                iteration boundary
              </li>
              <li>Autorun in 5 projects — switched off</li>
              <li>3 schedules — skipped, not queued</li>
              <li>Branches, artifacts and memory — untouched</li>
            </ul>
            <label class="dispatch-handoff">
              <input
                type="checkbox"
                checked={handoff()}
                onChange={(event) => setHandoff(event.currentTarget.checked)}
              />{" "}
              Let each agent write its handoff first
            </label>
            <p class="dispatch-handoff-note">
              {handoff()
                ? "Up to 30 seconds each. A successor starts from the payload instead of re-deriving it."
                : "Immediate. Work in flight is discarded and the next agent starts from the transcript."}
            </p>
            <footer>
              <span>
                Reversible for {STOP_ALL_RESTORE_MINUTES} minutes — the handoffs
                are kept.
              </span>
              <Button variant="ghost" onClick={() => setStopOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={() => {
                  setStopOpen(false);
                  setStopped(true);
                }}
              >
                Stop all — {STOP_ALL_AGENT_COUNT} agents
              </Button>
            </footer>
          </section>
        </div>
      </Show>
    </div>
  );
}

function SchedulesView() {
  const [permissionPosture, setPermissionPosture] =
    createSignal<PermissionPosture>("bypass");

  return (
    <div class="dispatch-view" data-testid="dispatch-schedules">
      <header class="dispatch-header">
        <DispatchTabs active="schedules" />
        <span class="dispatch-header-note">
          6 schedules · 248 fired · 17 skipped
        </span>
        <Button>New schedule</Button>
      </header>
      <div class="dispatch-scroll">
        <section class="dispatch-section schedule-intro">
          <h1>Schedules</h1>
          <p>
            A cron expression fires a workflow.{" "}
            <strong>locusd outlives the window</strong> — a schedule that only
            fires while the app happens to be open is not a schedule.
          </p>
          <div class="schedule-create">
            <strong>0 2 * * *</strong>
            <span>every day at 02:00 · America/Chicago</span>
            <span data-testid="schedule-outcome">
              <small
                id="schedule-overlap-note"
                data-testid="schedule-overlap-note"
              >
                Overlap is skipped, never queued. A job that runs longer than
                its own interval does not build a backlog.
              </small>
            </span>
          </div>
        </section>
        <section class="schedule-builder" data-testid="schedule-builder">
          <h2>Start work</h2>
          <fieldset>
            <legend>What runs</legend>
            <label>
              <input type="radio" name="schedule-run-mode" checked /> Project
            </label>
            <span>
              Runs every active agent on its own assignment; agents with nothing
              assigned are skipped.
            </span>
            <label>
              <input type="radio" name="schedule-run-mode" /> Custom
            </label>
            <span>
              Agent · Harness · Project · optional spec · optional prompt
            </span>
          </fieldset>
          <fieldset>
            <legend>
              Guardrails <small>optional override</small>
            </legend>
            <div class="schedule-guardrail-pills">
              <code>preset: default</code>
              <code>max iterations: fall through</code>
              <code>change ceiling: fall through</code>
              <code>files touched: fall through</code>
              <code>network: fall through</code>
              <code>token budget: fall through</code>
            </div>
            <span>
              Anything left unset falls through to Settings → Guardrails for
              #project. A ceiling reached here stops the run and splits it; it
              does not fail.
            </span>
          </fieldset>
          <fieldset data-testid="dispatch-permission-mode">
            <legend>Permissions</legend>
            <label>
              <input
                type="radio"
                name="dispatch-permission-mode"
                value="bypass"
                checked={permissionPosture() === "bypass"}
                onChange={() => setPermissionPosture("bypass")}
              />{" "}
              Bypass (default)
            </label>
            <span>
              The harness gate stays off. An unexpected permission request is
              recorded as an alarm.
            </span>
            <label>
              <input
                type="radio"
                name="dispatch-permission-mode"
                value="gated"
                checked={permissionPosture() === "gated"}
                onChange={() => setPermissionPosture("gated")}
              />{" "}
              Gated approval
            </label>
            <span data-testid="dispatch-permission-consequence">
              {permissionPosture() === "gated"
                ? "Protected requests wait for a human action and can be resolved after replay."
                : "This job starts unattended; protected requests raise a bypass alarm."}
            </span>
          </fieldset>
          <fieldset>
            <legend>When</legend>
            <label>
              <input type="radio" name="schedule-when" checked /> Run once, now
            </label>
            <label>
              <input type="radio" name="schedule-when" /> On a schedule
            </label>
            <label>
              <input type="radio" name="schedule-when" /> Hold
            </label>
            <div class="schedule-presets">
              <button type="button">Hourly</button>
              <button type="button">Nightly</button>
              <button type="button">Weekdays 09:00</button>
              <button type="button">Once at a time I pick</button>
            </div>
          </fieldset>
          <p class="schedule-builder-note">
            A prompt produces a run and an artifact, but no board task — nothing
            reaches the board without a plan.{" "}
            <strong>Overlap is skipped, never queued.</strong>
          </p>
        </section>
        <aside class="schedule-warning">
          <strong>
            Nightly wiki reconcile has skipped 11 of its last 14 firings
          </strong>
          <span>
            Overlap is visible so a schedule that stops running is not silent.
          </span>
        </aside>
        <section class="schedule-cards" data-testid="schedule-cards">
          <For each={SCHEDULES}>
            {(schedule) => (
              <article class="schedule-card" data-schedule={schedule.id}>
                <header>
                  <i data-live={schedule.enabled ? "true" : undefined} />
                  <strong>{schedule.name}</strong>
                  <span>{schedule.enabled ? "live" : "paused"}</span>
                </header>
                <code>{schedule.cron}</code>
                <span>{schedule.cadence}</span>
                <p>{schedule.workflow}</p>
                <footer>
                  <span
                    class={`schedule-last schedule-last-${schedule.last.split(" ")[0]}`}
                  >
                    {schedule.last}
                  </span>
                  <span>
                    Skipped <strong>{schedule.skipped}</strong>
                  </span>
                </footer>
              </article>
            )}
          </For>
        </section>
        <section class="schedule-executions" data-testid="schedule-executions">
          <header>
            <h2>Executions</h2>
            <span>
              recorded with their verify result — green or red, never merely
              “finished”
            </span>
          </header>
          <table>
            <thead>
              <tr>
                <th>Fired</th>
                <th>Schedule</th>
                <th>Result</th>
                <th>Duration</th>
                <th>Evidence</th>
              </tr>
            </thead>
            <tbody>
              <For each={SCHEDULE_EXECUTIONS}>
                {(execution) => (
                  <tr>
                    <td>{execution.firedAt}</td>
                    <td>{execution.schedule}</td>
                    <td
                      class={`verify-${execution.result === "passed" ? "ok" : execution.result === "failed" ? "bad" : "skipped"}`}
                    >
                      {execution.result}
                    </td>
                    <td>{execution.duration}</td>
                    <td>{execution.evidence}</td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </section>
      </div>
    </div>
  );
}

function RunsFixtureView() {
  return (
    <div class="dispatch-view" data-testid="dispatch-runs">
      <header class="dispatch-header">
        <DispatchTabs active="runs" />
        <span class="dispatch-header-note">
          Every run, scheduled or not · a schedule is just one way a run starts
        </span>
      </header>
      <div class="dispatch-runs-controls" data-testid="dispatch-pause-controls">
        <span>Search every run — a path, a tool name, an event verb</span>
        <span>
          Today · 7d · <strong>30d</strong>
        </span>
        <span>612 runs · 300 sessions · 4 projects</span>
      </div>
      <section class="dispatch-runs-table" data-testid="dispatch-runs-table">
        <h2>Runs (612)</h2>
        <div
          class="dispatch-verify-vocabulary"
          data-testid="runs-verify-vocabulary"
        >
          <span>Verify</span>
          <For each={VERIFY_VOCABULARY}>
            {(status) => (
              <code
                class={`verify-vocabulary-${status.replace(/[^a-z]+/g, "-")}`}
              >
                {status}
              </code>
            )}
          </For>
        </div>
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Harness</th>
              <th>Project</th>
              <th>repo</th>
              <th>Agent</th>
              <th>role</th>
              <th>Model resolved</th>
              <th>Events</th>
              <th>Errors</th>
              <th>Tokens</th>
              <th>Verify</th>
              <th>Id</th>
            </tr>
          </thead>
          <tbody>
            <For each={RUN_FIXTURE_ROWS}>
              {(run) => (
                <tr>
                  <td>{run.at.slice(0, 16).replace("T", " ")}</td>
                  <td>{run.harness}</td>
                  <td>{run.project}</td>
                  <td>core</td>
                  <td>{run.agent}</td>
                  <td>{run.role}</td>
                  <td>{run.model}</td>
                  <td>{run.events.toLocaleString("en-US")}</td>
                  <td class={run.errors > 0 ? "verify-bad" : ""}>
                    {run.errors || "—"}
                  </td>
                  <td>
                    {run.tokens === null ? (
                      <span class="unknown">unknown</span>
                    ) : (
                      `${(run.tokens / 1000).toFixed(1)}k`
                    )}
                  </td>
                  <td
                    class={`verify-${run.status === "passed" ? "ok" : run.status === "failed" ? "bad" : "skipped"}`}
                  >
                    {run.status}
                  </td>
                  <td>{run.id}</td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </section>
    </div>
  );
}

/** Dispatch's three durable-queue fixture routes. */
export function DispatchView(props: DispatchViewProps) {
  if (props.tab === "schedules") return <SchedulesView />;
  if (props.tab === "runs") return <RunsFixtureView />;
  return <AutorunView />;
}

export default DispatchView;
