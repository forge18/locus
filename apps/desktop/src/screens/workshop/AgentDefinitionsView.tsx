import { For, Match, Show, createSignal, onMount } from "solid-js";
import {
  fetchAgentDefFromCore,
  fetchAgentDefsFromCore,
} from "../../data/agent-defs";
import { dataProvider } from "../../data/provider";
import { CapabilityPolicyPanel } from "./CapabilityPolicyPanel";
import "./capability-policy.css";
import type {
  CoreAgentDefinition,
  AgentDefSummary,
} from "../../data/agent-defs";
import type { Envelope } from "../../data/envelope";
import { InlineError } from "../../ui/InlineError";

export function AgentDefinitionsView() {
  const [definitions, setDefinitions] = createSignal<
    Envelope<AgentDefSummary[]>
  >({
    status: "loading",
  });
  const [selected, setSelected] = createSignal<Envelope<CoreAgentDefinition>>();
  const definitionRows = () => {
    const state = definitions();
    return state.status === "ready" ? state.data : [];
  };
  const definitionError = () => {
    const state = definitions();
    return state.status === "failed" ? state.error.message : "";
  };
  const selectedError = () => {
    const state = selected();
    return state?.status === "failed" ? state.error.message : "";
  };
  const selectedName = () => {
    const state = selected();
    return state?.status === "ready" ? state.data.name : "Agent definition";
  };
  const selectedBody = () => {
    const state = selected();
    return state?.status === "ready" ? state.data.body : "";
  };

  onMount(() => {
    void fetchAgentDefsFromCore()
      .then(setDefinitions)
      .catch((cause) =>
        setDefinitions({
          status: "failed",
          error: { command: "agent_defs_list", message: String(cause) },
        }),
      );
  });

  const openDefinition = (name: string) => {
    setSelected({ status: "loading" });
    void fetchAgentDefFromCore(name)
      .then(setSelected)
      .catch((cause) =>
        setSelected({
          status: "failed",
          error: { command: "agent_def", message: String(cause) },
        }),
      );
  };

  return (
    <div data-testid="workshop-agents" class="agents-screen">
      <h1>Agent definitions</h1>
      <Show when={definitions().status === "loading"}>
        <p data-testid="workshop-agents-loading">Loading agent definitions…</p>
      </Show>
      <Show when={definitions().status === "failed"}>
        <InlineError
          cause={definitionError()}
          next="Agent definitions could not be loaded from the store."
        />
      </Show>
      <Show when={definitions().status === "empty"}>
        <p data-testid="workshop-agents-empty">
          No agent definitions are persisted.
        </p>
      </Show>
      <Show when={definitions().status === "ready"}>
        <div data-testid="workshop-agent-definitions">
          <For each={definitionRows()}>
            {(definition) => (
              <button
                type="button"
                onClick={() => openDefinition(definition.name)}
              >
                {definition.name} · v{definition.version}
              </button>
            )}
          </For>
        </div>
      </Show>
      <Show when={dataProvider().kind === "live"}>
        <CapabilityPolicyPanel />
      </Show>
      <Show when={selected()}>
        <Match when={selected()?.status === "loading"}>
          <p>Loading definition…</p>
        </Match>
        <Match when={selected()?.status === "failed"}>
          <InlineError
            cause={selectedError()}
            next="The selected definition could not be loaded."
          />
        </Match>
        <Match when={selected()?.status === "ready"}>
          <article data-testid="workshop-agent-definition-detail">
            <h2>{selectedName()}</h2>
            <pre>{selectedBody()}</pre>
          </article>
        </Match>
      </Show>
    </div>
  );
}
