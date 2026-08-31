import { createSignal, For, Show, onMount } from "solid-js";
import { isTauri } from "@tauri-apps/api/core";
import { InlineError } from "../../ui/InlineError";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Icon } from "../../ui/Icon";
import {
  BUDGET_NOTE,
  COMPILED,
  COMPILED_NOTE,
  OPERAND_NOTE,
  PRESET_NOTE,
  WAITING_NOTE,
  useCanvas,
  useExpression,
  useGuardrails,
  useOperands,
  usePalette,
  fetchWorkflowDefinitions,
  usePresets,
  type WorkflowDefinitionSummary,
} from "../../data/workflow";
import type { Envelope } from "../../data/envelope";
import { WorkflowCanvas } from "../../workflow-canvas/WorkflowCanvas";

function LiveWorkflowList(props: { projectId?: string }) {
  const [definitions, setDefinitions] = createSignal<Envelope<WorkflowDefinitionSummary[]>>({
    status: "loading",
  });
  const rows = () => {
    const envelope = definitions();
    return envelope.status === "ready" ? envelope.data : [];
  };
  const error = () => {
    const envelope = definitions();
    return envelope.status === "failed" ? envelope.error : undefined;
  };

  onMount(() => {
    if (!props.projectId) {
      setDefinitions({
        status: "failed",
        error: { command: "workflow_definitions", message: "no project is selected" },
      });
      return;
    }
    void fetchWorkflowDefinitions(props.projectId).then(setDefinitions);
  });

  return (
    <div class="wf" data-testid="workflow" data-live-state="ready">
      <header class="ws-head">
        <span class="ws-title">Workflow definitions</span>
        <span class="ws-note">Immutable project-owned versions</span>
      </header>
      <Show when={definitions().status === "loading"}>
        <p data-testid="workflow-loading">Loading workflows…</p>
      </Show>
      <Show when={error()}>
        <InlineError
          cause={error()!.message}
          next="Workflow definitions could not be loaded from the store."
        />
      </Show>
      <Show when={definitions().status === "empty"}>
        <p data-testid="workflow-empty">No workflow definitions are persisted for this project.</p>
      </Show>
      <Show when={definitions().status === "ready"}>
        <div data-testid="workflow-definitions">
          <For each={rows()}>
            {(definition) => (
              <article data-testid={`workflow-definition-${definition.id}`}>
                <strong>{definition.name}</strong>
                <span>v{definition.version}</span>
              </article>
            )}
          </For>
        </div>
        <p data-testid="workflow-detail-unavailable">
          Graph, governance, and compile details are not yet exposed by the live desktop contract.
        </p>
      </Show>
    </div>
  );
}

/**
 * Layout and inspector on fixture data. Real graph editing, compile and the live
 * overlay are `workflow-canvas` at M4, gated by Spike 3.
 */
export function WorkflowView(props: { projectId?: string } = {}) {
  if (isTauri()) return <LiveWorkflowList projectId={props.projectId} />;
  const canvas = useCanvas();
  const [expandedPreset, setExpandedPreset] = createSignal<string>();

  return (
    <div class="wf" data-testid="workflow">
      <aside class="wf-palette" data-testid="wf-palette">
        <span class="wf-section">Nodes</span>
        <For each={usePalette()}>
          {(node) => (
            <div
              class={[
                "wf-chip",
                node.tone !== "default" ? `wf-chip-${node.tone}` : "",
              ]
                .filter(Boolean)
                .join(" ")}
              data-testid={`wf-chip-${node.kind}`}
              data-tone={node.tone}
              draggable="true"
            >
              <Icon
                name="dots-six-vertical"
                size={10}
                style={{ color: "var(--text-muted)" }}
              />
              <Icon name={node.icon} size={11} />
              {node.label}
              <Show when={node.required}>
                <span
                  class="wf-chip-req"
                  data-testid={`wf-chip-req-${node.kind}`}
                >
                  req
                </span>
              </Show>
            </div>
          )}
        </For>

        <span class="wf-section" data-testid="wf-presets-title">
          Presets
        </span>
        <For each={usePresets()}>
          {(preset) => (
            <button
              type="button"
              class="wf-preset"
              data-testid={`wf-preset-${preset.name.replace(/\W+/g, "-")}`}
              aria-expanded={expandedPreset() === preset.name}
              onClick={() => setExpandedPreset(preset.name)}
            >
              {preset.name}
              <div class="wf-preset-note">{preset.note}</div>
            </button>
          )}
        </For>
        <Show when={expandedPreset()}>
          <div
            class="wf-preset-expanded"
            data-testid="wf-preset-expanded"
            data-preset={expandedPreset()}
          >
            <span>ordinary nodes</span>
            <span>pick</span>
            <span>act</span>
            <span>validate</span>
            <span>commit</span>
            <span>reset</span>
          </div>
        </Show>
        <span class="wf-preset-note" data-testid="wf-preset-note">
          {PRESET_NOTE}
        </span>
      </aside>

      <WorkflowCanvas
        nodes={canvas.nodes}
        edges={canvas.edges}
        loop={canvas.loop}
        events={canvas.events}
      />

      <aside class="wf-inspector" data-testid="wf-inspector">
        <FixtureNotice
          surface="Workflow canvas"
          command='invoke("workflow_graph")'
        />
        <div class="wf-inspector-body">
          <span class="wf-section" data-testid="wf-inspector-title">
            Condition · verify.passed
          </span>

          <For each={useExpression()}>
            {(clause, i) => (
              <>
                <Show when={i() > 0}>
                  <span class="clause-joiner" data-testid="clause-joiner">
                    and
                  </span>
                </Show>
                <div class="clause" data-testid={`clause-${i()}`}>
                  <span class="clause-field">{clause.operand}</span>
                  <span class="clause-field">{clause.operator}</span>
                  <span class="clause-field">{clause.value}</span>
                </div>
              </>
            )}
          </For>
          <button type="button" class="clause-add" data-testid="clause-add">
            + add clause
          </button>

          <div class="compiled" data-testid="wf-compiled">
            <div class="compiled-expr" data-testid="wf-compiled-expr">
              {COMPILED}
            </div>
            <div class="compiled-note" data-testid="wf-compiled-note">
              {COMPILED_NOTE}
            </div>
          </div>

          <span class="wf-section" data-testid="wf-operands-title">
            Operands — every one is a column
          </span>
          <div class="operands" data-testid="wf-operands">
            <For each={useOperands()}>
              {(operand) => (
                <span
                  class="operand"
                  data-testid={`operand-${operand.replace(/\W+/g, "-")}`}
                >
                  {operand}
                </span>
              )}
            </For>
          </div>
          <span class="wf-preset-note" data-testid="wf-operand-note">
            {OPERAND_NOTE}
          </span>

          <span class="wf-section" data-testid="wf-guardrails-title">
            Guardrails
          </span>
          <For each={useGuardrails()}>
            {(guardrail) => (
              <div
                class="guardrail-row"
                data-testid={`guardrail-${guardrail.key}`}
              >
                {guardrail.label}
                <Show
                  when={guardrail.kind === "stepper"}
                  fallback={
                    <Show
                      when={guardrail.kind === "toggle"}
                      fallback={
                        <span
                          class="guardrail-value"
                          data-testid={`guardrail-value-${guardrail.key}`}
                        >
                          {guardrail.value}
                        </span>
                      }
                    >
                      <span
                        class="toggle"
                        data-testid={`guardrail-toggle-${guardrail.key}`}
                        data-on={guardrail.value === "on" ? "true" : "false"}
                      />
                    </Show>
                  }
                >
                  <span
                    class="stepper"
                    data-testid={`guardrail-stepper-${guardrail.key}`}
                  >
                    <button
                      type="button"
                      aria-label={`Decrease ${guardrail.label}`}
                    >
                      −
                    </button>
                    <span
                      class="guardrail-value"
                      data-testid={`guardrail-value-${guardrail.key}`}
                    >
                      {guardrail.value}
                    </span>
                    <button
                      type="button"
                      aria-label={`Increase ${guardrail.label}`}
                    >
                      +
                    </button>
                  </span>
                </Show>
              </div>
            )}
          </For>
          <span class="wf-preset-note" data-testid="wf-budget-note">
            {BUDGET_NOTE}
          </span>
          <span class="wf-preset-note" data-testid="wf-waiting-note">
            {WAITING_NOTE}
          </span>
        </div>

        <footer class="wf-inspector-foot" data-testid="wf-inspector-foot">
          <span data-testid="wf-autosave">saved 2s ago</span>
        </footer>
      </aside>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default WorkflowView;
