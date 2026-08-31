import {
      For,
      Match,
      Show,
      Switch,
      createEffect,
      createMemo,
      createSignal,
      on,
      onCleanup,
      onMount,
} from "solid-js";
import { Breadcrumb } from "./Breadcrumb";
import { Message } from "./Message";
import { PlanList } from "./PlanList";
import { Recommendation } from "./Recommendation";
import { ScopeDecision } from "./ScopeDecision";
import { PlanSpecView } from "./PlanSpecView";
import { PlanTasksView } from "./PlanTasksView";
import { Icon } from "../../ui/Icon";
import { InlineError } from "../../ui/InlineError";
import { Tag } from "../../ui/Tag";
import { Button } from "../../ui/Button";
import { isTauri } from "@tauri-apps/api/core";
import type { NavStore } from "../../nav";
import type { Envelope } from "../../data/envelope";
import {
      ACP_LABEL,
      createPlan,
      fetchPlans,
      useDefaultPlanId,
      usePlanConversation,
      usePlanLiveLine,
      usePlanOutputs,
      subscribePlanConversationFromCore,
      usePlanRecommendation,
      usePlanScopeDecision,
      usePlans,
      type PlanSummary,
} from "../../data/plan";

export const PLAN_STAGE_LABELS = [
      "Inputs",
      "Orient",
      "Converse",
      "Synthesis",
      "Recommend",
      "Decompose",
      "Approved",
] as const;

type PlanStage = (typeof PLAN_STAGE_LABELS)[number];

function UnavailablePanel(props: { title: string; detail: string }) {
      return (
            <section class="plan-stage-panel" data-testid="plan-unavailable">
                  <h2>{props.title}</h2>
                  <p>{props.detail}</p>
            </section>
      );
}

function StagePanel(props: {
      stage: PlanStage;
      newPlan: boolean;
      unavailable: boolean;
      goal: string;
      onGoal: (goal: string) => void;
      busy: boolean;
      onStart: () => void;
      onNewPlan: () => void;
}) {
      if (props.unavailable && props.stage !== "Inputs")
            return (
                  <UnavailablePanel
                        title={`${props.stage} is unavailable`}
                        detail="This stage has no persisted desktop contract yet. The live plan list is shown without invented stage data."
                  />
            );
      if (props.stage === "Inputs") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-inputs-stage"
                  >
                        <h2>What should this plan add?</h2>
                        <p>
                              One goal per plan. The conversation will say if
                              this should split.
                        </p>
                        <label>
                              Goal
                              <textarea
                                    data-testid="plan-goal"
                                    value={props.goal}
                                    placeholder="Describe the outcome this plan should produce"
                                    onInput={(event) =>
                                          props.onGoal(
                                                event.currentTarget.value,
                                          )
                                    }
                              />
                        </label>
                        <label>
                              Project
                              <Show
                                    when={!props.unavailable}
                                    fallback={
                                          <span data-testid="plan-active-project">
                                                Active project · live scope
                                          </span>
                                    }
                              >
                                    <select
                                          data-testid="plan-project"
                                          value="tapestry"
                                    >
                                          <option value="tapestry">
                                                #tapestry
                                          </option>
                                          <option value="loom-db">
                                                #loom-db
                                          </option>
                                          <option value="weaver">
                                                #weaver
                                          </option>
                                    </select>
                              </Show>
                        </label>
                        <div data-testid="plan-attached-repos">
                              <strong>Attached repositories</strong>
                              <span>
                                    {props.unavailable
                                          ? "Loaded from the active project"
                                          : "core · desktop"}
                              </span>
                        </div>
                        <Button
                              variant="primary"
                              data-testid="start-planning"
                              disabled={props.busy}
                              onClick={props.onStart}
                        >
                              Start planning
                        </Button>
                        <p class="plan-stage-note">
                              One goal per plan. If this turns out to be two
                              things, the conversation will say so and you can
                              split it there.
                        </p>
                  </section>
            );
      }
      if (props.stage === "Orient") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-orient-stage"
                  >
                        <h2>Orient</h2>
                        <p>
                              Indexing the project before Converse becomes
                              reachable.
                        </p>
                        <ul data-testid="plan-orient-progress">
                              <li data-state="complete">symbols · indexed</li>
                              <li data-state="complete">
                                    call graph · indexed
                              </li>
                              <li data-state="complete">history · indexed</li>
                        </ul>
                  </section>
            );
      }
      if (props.stage === "Synthesis") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-synthesis-stage"
                  >
                        <h2>Synthesis</h2>
                        <div data-testid="synthesis-pass-one">
                              Pass 1 · requirements drafted
                        </div>
                        <div data-testid="synthesis-pass-two">
                              Pass 2 · unsupported clauses removed
                        </div>
                        <strong data-testid="synthesis-open-count">
                              open[2] carried to Recommend
                        </strong>
                  </section>
            );
      }
      if (props.stage === "Recommend") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-recommend-stage"
                  >
                        <h2>Recommend</h2>
                        <p>spec.md · 14 requirements · version 7</p>
                        <strong>confidence 0.62 · open[2]</strong>
                        <p>
                              Save &amp; re-synthesise only the requirements you
                              changed.
                        </p>
                  </section>
            );
      }
      if (props.stage === "Decompose") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-decompose-stage"
                  >
                        <h2>Decompose</h2>
                        <p>
                              Choose what becomes a card. Harness selection
                              enables model and effort overrides.
                        </p>
                        <span data-testid="decompose-routing-default">
                              auto-route
                        </span>
                  </section>
            );
      }
      if (props.stage === "Approved") {
            return (
                  <section
                        class="plan-stage-panel"
                        data-testid="plan-approved-stage"
                  >
                        <h2>Approved — 8 cards on the board</h2>
                        <div
                              class="plan-approved-stats"
                              data-testid="approved-stat-cards"
                        >
                              <span>Questions · 14 / 12 / 2</span>
                              <span>Requirements · 14</span>
                              <span>Confidence · 0.62</span>
                              <span>Cards · 8</span>
                        </div>
                        <section data-testid="approved-stage-log">
                              <h3>What happened</h3>
                              <p>
                                    Inputs → Orient → Converse → Synthesis →
                                    Recommend → Decompose → Approved
                              </p>
                        </section>
                        <table data-testid="approved-cards">
                              <thead>
                                    <tr>
                                          <th>Id</th>
                                          <th>Title</th>
                                          <th>Workflow · Harness</th>
                                    </tr>
                              </thead>
                              <tbody>
                                    <tr>
                                          <td>T-01</td>
                                          <td>
                                                Confidence column on memory.fact
                                          </td>
                                          <td>build · claude</td>
                                    </tr>
                              </tbody>
                        </table>
                        <Button
                              variant="primary"
                              data-testid="approved-new-plan"
                              onClick={props.onNewPlan}
                        >
                              Start a new plan
                        </Button>
                        <Button
                              variant="secondary"
                              data-testid="approved-open-board"
                        >
                              Open the board
                        </Button>
                  </section>
            );
      }
      return null;
}

/**
 * A guided conversation that produces a reviewable plan. Nothing reaches the board
 * until one approval at the end, which is why the recommendation has to be legible
 * enough to approve honestly rather than just clickable.
 */
export interface PlanViewProps {
      nav?: NavStore;
}

export function PlanView(props: PlanViewProps = {}) {
      const liveMode = isTauri();
      const fixturePlans = usePlans();
      const [planEnvelope, setPlanEnvelope] = createSignal<
            Envelope<PlanSummary[]>
      >(
            liveMode
                  ? { status: "loading" }
                  : { status: "ready", data: fixturePlans },
      );
      const plans = createMemo(() => {
            const envelope = planEnvelope();
            return envelope.status === "ready" ? envelope.data : [];
      });
      const [selectedId, setSelectedId] = createSignal(
            liveMode ? "" : useDefaultPlanId(),
      );
      const [tab, setTab] = createSignal<"conversation" | "spec" | "tasks">(
            "conversation",
      );
      const [plansOpen, setPlansOpen] = createSignal(true);
      const [outputsOpen, setOutputsOpen] = createSignal(true);
      const [newPlan, setNewPlan] = createSignal(false);
      const [goalDraft, setGoalDraft] = createSignal("");
      const [creatingPlan, setCreatingPlan] = createSignal(false);
      const selected = createMemo<PlanSummary | undefined>(
            () => plans().find((p) => p.id === selectedId()) ?? plans()[0],
      );
      const [stage, setStage] = createSignal<PlanStage>("Inputs");
      const stageIndex = () => PLAN_STAGE_LABELS.indexOf(stage());
      const moveStage = (delta: -1 | 1) => {
            const next = stageIndex() + delta;
            if (next >= 0 && next < PLAN_STAGE_LABELS.length)
                  setStage(PLAN_STAGE_LABELS[next]);
      };
      const selectPlan = (id: string) => {
            setNewPlan(false);
            setSelectedId(id);
            const next = plans().find((plan) => plan.id === id);
            if (next) setStage(next.step);
      };
      const startNewPlan = () => {
            setNewPlan(true);
            setSelectedId("");
            setGoalDraft("");
            setStage("Inputs");
            setTab("conversation");
      };
      const startPlanning = () => {
            if (!liveMode) {
                  setNewPlan(false);
                  setStage("Orient");
                  return;
            }
            const projectId = props.nav?.params().project;
            const goal = goalDraft().trim();
            if (!projectId || !goal) return;
            setCreatingPlan(true);
            const title = goal.split("\\n", 1)[0].slice(0, 80);
            void createPlan(projectId, title, goal)
                  .then((envelope) => {
                        if (envelope.status === "ready") {
                              setPlanEnvelope((current) =>
                                    current.status === "ready"
                                          ? {
                                                  status: "ready",
                                                  data: [
                                                        ...current.data,
                                                        envelope.data,
                                                  ],
                                            }
                                          : {
                                                  status: "ready",
                                                  data: [envelope.data],
                                            },
                              );
                              setSelectedId(envelope.data.id);
                              setNewPlan(false);
                              setStage(envelope.data.step);
                        } else {
                              setPlanEnvelope(envelope);
                        }
                  })
                  .finally(() => setCreatingPlan(false));
      };

      const [messages, setMessages] = createSignal(
            liveMode ? [] : usePlanConversation(),
      );
      const [streamError, setStreamError] = createSignal<string | null>(null);
      const outputs = usePlanOutputs();

      async function refreshPlans() {
            if (!liveMode) return;
            const projectId = props.nav?.params().project;
            if (!projectId) {
                  setPlanEnvelope({
                        status: "failed",
                        error: {
                              command: "plans_list",
                              message: "no project is selected",
                        },
                  });
                  return;
            }
            const envelope = await fetchPlans(projectId);
            setPlanEnvelope(envelope);
            if (envelope.status === "ready") {
                  const current = selectedId();
                  setSelectedId(
                        envelope.data.some((plan) => plan.id === current)
                              ? current
                              : (envelope.data[0]?.id ?? ""),
                  );
                  const first = envelope.data[0];
                  if (first) setStage(first.step);
            }
      }

      createEffect(
            on(
                  () => props.nav?.params().project,
                  () => {
                        void refreshPlans();
                  },
                  { defer: true },
            ),
      );

      createEffect(() => {
            const plan = selected();
            if (plan && !newPlan()) setStage(plan.step);
      });

      onMount(() => {
            let stopped = false;
            let detach = () => undefined;
            onCleanup(() => {
                  stopped = true;
                  detach();
            });
            void subscribePlanConversationFromCore((message) => {
                  if (!stopped) {
                        setMessages((current) =>
                              current.some((item) => item.id === message.id)
                                    ? current
                                    : [...current, message],
                        );
                  }
            })
                  .then((channel) => {
                        if (!channel) return;
                        detach = () => {
                              channel.onmessage = () => undefined;
                        };
                        if (stopped) detach();
                  })
                  .catch((e) => {
                        if (!stopped)
                              setStreamError(
                                    e instanceof Error ? e.message : String(e),
                              );
                  });
      });

      const planError = createMemo(() => {
            const envelope = planEnvelope();
            return envelope.status === "failed" ? envelope.error : null;
      });

      return (
            <div class="plan-workspace" data-testid="plan">
                  <Switch>
                        <Match when={planEnvelope().status === "loading"}>
                              <UnavailablePanel
                                    title="Loading plans…"
                                    detail="Reading the selected project's plans from the live store."
                              />
                        </Match>
                        <Match when={planEnvelope().status === "empty"}>
                              <UnavailablePanel
                                    title="No plans yet"
                                    detail="This project has no persisted plans. Start planning when the planning mutation contract is available."
                              />
                        </Match>
                        <Match when={planError()}>
                              <InlineError
                                    cause={planError()!.message}
                                    next="The plans list could not be loaded from the live store."
                              />
                        </Match>
                        <Match
                              when={
                                    planEnvelope().status === "ready" &&
                                    plans().length > 0
                              }
                        >
                              <>
                                    <Show when={streamError()}>
                                          <div data-testid="plan-stream-error">
                                                <InlineError
                                                      cause={streamError()!}
                                                      next="Live conversation updates are unavailable; persisted plan data remains visible."
                                                />
                                          </div>
                                    </Show>
                                    <header class="plan-workspace-head">
                                          <button
                                                type="button"
                                                class="plan-workspace-toggle"
                                                data-testid="toggle-plans"
                                                onClick={() =>
                                                      setPlansOpen(
                                                            (open) => !open,
                                                      )
                                                }
                                          >
                                                All plans{" "}
                                                <span class="mono">
                                                      {plans().length}
                                                </span>
                                          </button>
                                          <div
                                                class="plan-workspace-tabs"
                                                data-testid="plan-workspace-tabs"
                                                role="tablist"
                                                aria-label="Plan view"
                                          >
                                                <button
                                                      type="button"
                                                      role="tab"
                                                      data-testid="plan-tab-conversation"
                                                      aria-selected={
                                                            tab() ===
                                                            "conversation"
                                                      }
                                                      onClick={() =>
                                                            setTab(
                                                                  "conversation",
                                                            )
                                                      }
                                                >
                                                      Conversation
                                                </button>
                                                <button
                                                      type="button"
                                                      role="tab"
                                                      data-testid="plan-tab-spec"
                                                      aria-selected={
                                                            tab() === "spec"
                                                      }
                                                      onClick={() =>
                                                            setTab("spec")
                                                      }
                                                >
                                                      Spec
                                                </button>
                                                <button
                                                      type="button"
                                                      role="tab"
                                                      data-testid="plan-tab-tasks"
                                                      aria-selected={
                                                            tab() === "tasks"
                                                      }
                                                      onClick={() =>
                                                            setTab("tasks")
                                                      }
                                                >
                                                      Tasks &amp; cards
                                                </button>
                                          </div>
                                          <button
                                                type="button"
                                                class="plan-workspace-toggle"
                                                data-testid="toggle-outputs"
                                                onClick={() =>
                                                      setOutputsOpen(
                                                            (open) => !open,
                                                      )
                                                }
                                          >
                                                Outputs{" "}
                                                <span class="mono">4</span>
                                          </button>
                                    </header>
                                    <div class="plan-summary">
                                          <span
                                                class="plan-convo-title"
                                                data-testid="plan-title"
                                          >
                                                {selected()!.title}
                                          </span>
                                          <span>
                                                {selected()!.project} · started
                                                09:14
                                          </span>
                                          <Breadcrumb current={stage()} />
                                    </div>
                                    <div
                                          class="plan-stage-stepper"
                                          data-testid="plan-stage-stepper"
                                    >
                                          <button
                                                type="button"
                                                data-testid="plan-stage-back"
                                                disabled={stageIndex() === 0}
                                                onClick={() => moveStage(-1)}
                                          >
                                                Back
                                          </button>
                                          <span data-testid="plan-stage-step">
                                                Step {stageIndex() + 1} of 7 ·{" "}
                                                {stage()}
                                          </span>
                                          <button
                                                type="button"
                                                data-testid="plan-stage-next"
                                                disabled={
                                                      stageIndex() ===
                                                      PLAN_STAGE_LABELS.length -
                                                            1
                                                }
                                                onClick={() => moveStage(1)}
                                          >
                                                Next
                                          </button>
                                    </div>
                                    <nav
                                          class="plan-stage-strip"
                                          data-testid="plan-stage-strip"
                                          aria-label="Plan stages"
                                    >
                                          <For each={PLAN_STAGE_LABELS}>
                                                {(entry, index) => (
                                                      <button
                                                            type="button"
                                                            data-stage={entry}
                                                            aria-current={
                                                                  stage() ===
                                                                  entry
                                                                        ? "step"
                                                                        : undefined
                                                            }
                                                            onClick={() =>
                                                                  setStage(
                                                                        entry,
                                                                  )
                                                            }
                                                      >
                                                            {index() + 1}{" "}
                                                            {entry}
                                                      </button>
                                                )}
                                          </For>
                                    </nav>

                                    <div class="plan">
                                          <Show when={plansOpen()}>
                                                <PlanList
                                                      plans={plans()}
                                                      selectedId={selectedId()}
                                                      onSelect={selectPlan}
                                                      onNewPlan={startNewPlan}
                                                />
                                          </Show>

                                          <Show
                                                when={
                                                      tab() ===
                                                            "conversation" &&
                                                      stage() !== "Converse"
                                                }
                                          >
                                                <StagePanel
                                                      stage={stage()}
                                                      newPlan={newPlan()}
                                                      unavailable={
                                                            liveMode &&
                                                            stage() !== "Inputs"
                                                      }
                                                      goal={goalDraft()}
                                                      onGoal={setGoalDraft}
                                                      busy={creatingPlan()}
                                                      onStart={startPlanning}
                                                      onNewPlan={startNewPlan}
                                                />
                                          </Show>

                                          <Switch>
                                                <Match
                                                      when={
                                                            tab() ===
                                                            "conversation"
                                                      }
                                                >
                                                      <section
                                                            class="plan-convo"
                                                            data-testid="plan-conversation"
                                                      >
                                                            <header class="plan-convo-head">
                                                                  <span
                                                                        class="plan-stage-label"
                                                                        data-testid="plan-stage-progress"
                                                                  >
                                                                        Step{" "}
                                                                        {stageIndex() +
                                                                              1}{" "}
                                                                        of 7
                                                                  </span>
                                                                  <span class="plan-convo-title">
                                                                        {stage()}
                                                                  </span>
                                                                  <span class="plan-convo-running">
                                                                        <span class="live-dot pulse" />
                                                                        running
                                                                  </span>
                                                            </header>
                                                            <div
                                                                  class="plan-messages"
                                                                  data-testid="plan-messages"
                                                            >
                                                                  <For
                                                                        each={messages()}
                                                                  >
                                                                        {(
                                                                              message,
                                                                              i,
                                                                        ) => (
                                                                              <>
                                                                                    <Message
                                                                                          message={
                                                                                                message
                                                                                          }
                                                                                    />
                                                                                    <Show
                                                                                          when={
                                                                                                !liveMode &&
                                                                                                i() ===
                                                                                                      messages()
                                                                                                            .length -
                                                                                                            2
                                                                                          }
                                                                                    >
                                                                                          <ScopeDecision
                                                                                                decision={usePlanScopeDecision()}
                                                                                                onWiden={() => {}}
                                                                                                onKeepOut={() => {}}
                                                                                          />
                                                                                    </Show>
                                                                              </>
                                                                        )}
                                                                  </For>
                                                                  <Show
                                                                        when={
                                                                              liveMode &&
                                                                              messages()
                                                                                    .length ===
                                                                                    0
                                                                        }
                                                                  >
                                                                        <div data-testid="plan-conversation-unavailable">
                                                                              Live
                                                                              conversation
                                                                              details
                                                                              are
                                                                              not
                                                                              persisted
                                                                              by
                                                                              the
                                                                              current
                                                                              contract.
                                                                        </div>
                                                                  </Show>
                                                                  <div
                                                                        class="plan-live"
                                                                        data-testid="plan-live"
                                                                  >
                                                                        <span
                                                                              class="live-dot pulse"
                                                                              data-testid="plan-live-dot"
                                                                        />
                                                                        {liveMode
                                                                              ? "Waiting for live conversation events…"
                                                                              : usePlanLiveLine()}
                                                                  </div>
                                                            </div>
                                                            <footer
                                                                  class="plan-convo-footer"
                                                                  data-testid="plan-footer"
                                                            >
                                                                  <div
                                                                        class="plan-input"
                                                                        data-testid="plan-input"
                                                                  >
                                                                        Answer
                                                                        the
                                                                        interviewer…
                                                                        <span
                                                                              class="plan-caret blink"
                                                                              data-testid="plan-caret"
                                                                        >
                                                                              |
                                                                        </span>
                                                                  </div>
                                                                  <span
                                                                        class="plan-acp"
                                                                        data-testid="plan-acp"
                                                                  >
                                                                        {
                                                                              ACP_LABEL
                                                                        }
                                                                  </span>
                                                            </footer>
                                                      </section>
                                                </Match>
                                                <Match when={tab() === "spec"}>
                                                      {liveMode ? (
                                                            <UnavailablePanel
                                                                  title="Plan spec is unavailable"
                                                                  detail="Requirements are not yet exposed through a persisted desktop command."
                                                            />
                                                      ) : (
                                                            <PlanSpecView />
                                                      )}
                                                </Match>
                                                <Match when={tab() === "tasks"}>
                                                      {liveMode ? (
                                                            <UnavailablePanel
                                                                  title="Plan tasks are unavailable"
                                                                  detail="Decomposition and board-card outputs are not yet exposed through a persisted desktop command."
                                                            />
                                                      ) : (
                                                            <PlanTasksView />
                                                      )}
                                                </Match>
                                          </Switch>

                                          <Show when={outputsOpen()}>
                                                <Show
                                                      when={!liveMode}
                                                      fallback={
                                                            <aside
                                                                  class="plan-outputs"
                                                                  data-testid="plan-outputs"
                                                            >
                                                                  <UnavailablePanel
                                                                        title="Draft outputs are unavailable"
                                                                        detail="The current desktop contract persists the plan list only; spec, task, and recommendation outputs are not served as fixtures in a live window."
                                                                  />
                                                            </aside>
                                                      }
                                                >
                                                      <aside
                                                            class="plan-outputs"
                                                            data-testid="plan-outputs"
                                                      >
                                                            <span class="plan-outputs-title">
                                                                  Draft outputs
                                                            </span>
                                                            <section
                                                                  class="output-card"
                                                                  data-testid="output-spec"
                                                            >
                                                                  <div class="output-card-head">
                                                                        <Icon
                                                                              name="file-text"
                                                                              size={
                                                                                    12
                                                                              }
                                                                              style={{
                                                                                    color: "var(--text-secondary)",
                                                                              }}
                                                                        />
                                                                        <span class="mono">
                                                                              {
                                                                                    outputs
                                                                                          .spec
                                                                                          .name
                                                                              }
                                                                        </span>
                                                                        <button
                                                                              type="button"
                                                                              class="plan-output-edit"
                                                                              onClick={() =>
                                                                                    setTab(
                                                                                          "spec",
                                                                                    )
                                                                              }
                                                                        >
                                                                              Edit
                                                                        </button>
                                                                  </div>
                                                                  <For
                                                                        each={
                                                                              outputs
                                                                                    .spec
                                                                                    .lines
                                                                        }
                                                                  >
                                                                        {(
                                                                              line,
                                                                        ) => (
                                                                              <span class="output-line">
                                                                                    {
                                                                                          line
                                                                                    }
                                                                              </span>
                                                                        )}
                                                                  </For>
                                                            </section>
                                                            <section
                                                                  class="output-card"
                                                                  data-testid="output-tasks"
                                                            >
                                                                  <div class="output-card-head">
                                                                        <Icon
                                                                              name="list-checks"
                                                                              size={
                                                                                    12
                                                                              }
                                                                              style={{
                                                                                    color: "var(--text-secondary)",
                                                                              }}
                                                                        />
                                                                        tasks
                                                                        <button
                                                                              type="button"
                                                                              class="plan-output-edit"
                                                                              onClick={() =>
                                                                                    setTab(
                                                                                          "tasks",
                                                                                    )
                                                                              }
                                                                        >
                                                                              Edit
                                                                              &amp;
                                                                              decompose
                                                                        </button>
                                                                  </div>
                                                                  <ol
                                                                        class="output-tasks"
                                                                        data-testid="output-task-list"
                                                                  >
                                                                        <For
                                                                              each={
                                                                                    outputs.tasks
                                                                              }
                                                                        >
                                                                              {(
                                                                                    task,
                                                                              ) => (
                                                                                    <li>
                                                                                          {
                                                                                                task
                                                                                          }
                                                                                    </li>
                                                                              )}
                                                                        </For>
                                                                  </ol>
                                                            </section>
                                                            <section
                                                                  class="output-card"
                                                                  data-testid="output-tools"
                                                            >
                                                                  <div class="output-card-head">
                                                                        <Icon
                                                                              name="toolbox"
                                                                              size={
                                                                                    12
                                                                              }
                                                                              style={{
                                                                                    color: "var(--text-secondary)",
                                                                              }}
                                                                        />
                                                                        tool
                                                                        list
                                                                  </div>
                                                                  <div class="output-tools">
                                                                        <For
                                                                              each={
                                                                                    outputs.tools
                                                                              }
                                                                        >
                                                                              {(
                                                                                    tool,
                                                                              ) => (
                                                                                    <Tag variant="neutral">
                                                                                          {
                                                                                                tool
                                                                                          }
                                                                                    </Tag>
                                                                              )}
                                                                        </For>
                                                                        <For
                                                                              each={
                                                                                    outputs.newTools
                                                                              }
                                                                        >
                                                                              {(
                                                                                    tool,
                                                                              ) => (
                                                                                    <Tag
                                                                                          variant="outline"
                                                                                          data-testid={`new-tool-${tool.replace(/\W+/g, "")}`}
                                                                                    >
                                                                                          {
                                                                                                tool
                                                                                          }
                                                                                    </Tag>
                                                                              )}
                                                                        </For>
                                                                  </div>
                                                            </section>
                                                            <Recommendation
                                                                  recommendation={usePlanRecommendation()}
                                                                  onApprove={() =>
                                                                        setTab(
                                                                              "tasks",
                                                                        )
                                                                  }
                                                            />
                                                      </aside>
                                                </Show>
                                          </Show>
                                    </div>
                              </>
                        </Match>
                  </Switch>
            </div>
      );
}

/** Default export so the view can be code-split at the route boundary. */
export default PlanView;
