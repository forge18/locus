import { For, Show, createSignal, onMount } from "solid-js";
import { dataProvider } from "../../data/provider";
import type { Envelope } from "../../data/envelope";
import type { WorkItemProviderRecord } from "../../data/work-items";
import { InlineError } from "../../ui/InlineError";

export function ProvidersView() {
  const [providers, setProviders] = createSignal<
    Envelope<WorkItemProviderRecord[]>
  >({ status: "loading" });

  onMount(() => {
    void dataProvider()
      .query<WorkItemProviderRecord>("external_work_item_providers")
      .then(setProviders)
      .catch((cause) =>
        setProviders({
          status: "failed",
          error: {
            command: "external_work_item_providers",
            message: String(cause),
          },
        }),
      );
  });

  const rows = () => {
    const state = providers();
    return state.status === "ready" ? state.data : [];
  };
  const error = () => {
    const state = providers();
    return state.status === "failed" ? state.error.message : "";
  };

  return (
    <div data-testid="workshop-providers" class="providers-screen">
      <h1>Configured providers</h1>
      <Show when={providers().status === "loading"}>
        <p data-testid="workshop-providers-loading">Loading providers…</p>
      </Show>
      <Show when={providers().status === "failed"}>
        <InlineError
          cause={error()}
          next="Configured providers could not be loaded from the store."
        />
      </Show>
      <Show when={providers().status === "empty"}>
        <p data-testid="workshop-providers-empty">
          No providers are configured.
        </p>
      </Show>
      <Show when={providers().status === "ready"}>
        <div data-testid="workshop-provider-list">
          <For each={rows()}>
            {(provider) => (
              <article>
                <strong>{provider.label}</strong>
                <span>{provider.host}</span>
                <small>{provider.project}</small>
              </article>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
