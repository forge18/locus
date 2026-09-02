import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { dataProvider } from "../../data/provider";
import {
  fetchProviders,
  replaceProviderSecret,
  saveProvider,
  saveProviderModels,
  type ProviderModel,
  type ProviderRecord,
} from "../../data/providers";
import type { Envelope } from "../../data/envelope";
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";
import { Input } from "../../ui/Input";
import { Segmented } from "../../ui/Segmented";
import { Tag } from "../../ui/Tag";

const AUTHENTICATION_METHODS = [
  { value: "oauth", label: "OAuth" },
  { value: "api-key", label: "API key" },
  { value: "none", label: "None" },
];

type ProviderDraft = ProviderRecord;

function providerStatus(
  provider: ProviderRecord,
): "ok" | "warn" | "off" | "unconfigured" {
  if (provider.verificationStatus === "failed") return "off";
  if (provider.verificationStatus !== "verified") return "unconfigured";
  if (
    provider.verificationExpiresAt &&
    Date.parse(provider.verificationExpiresAt) <= Date.now()
  ) {
    return "warn";
  }
  return "ok";
}

function cloneProvider(provider: ProviderRecord): ProviderDraft {
  return {
    ...provider,
    models: provider.models.map((model) => ({ ...model })),
  };
}

export function ProvidersView() {
  const liveMode = dataProvider().kind === "live";
  const [providers, setProviders] = createSignal<Envelope<ProviderRecord[]>>({
    status: "loading",
  });
  const [selectedId, setSelectedId] = createSignal<string>();
  const [draft, setDraft] = createSignal<ProviderDraft>();
  const [saved, setSaved] = createSignal(false);
  const [saveError, setSaveError] = createSignal<string>();
  const [replacementSecret, setReplacementSecret] = createSignal("");
  const [modelSearch, setModelSearch] = createSignal("");

  onMount(() => {
    void fetchProviders().then((result) => {
      setProviders(result);
      if (result.status === "ready") {
        const first = result.data[0];
        if (first) {
          setSelectedId(first.id);
          setDraft(cloneProvider(first));
        }
      }
    });
  });

  const rows = () => {
    const state = providers();
    return state.status === "ready" ? state.data : [];
  };
  const loadError = () => {
    const state = providers();
    return state.status === "failed" ? state.error.message : "";
  };
  const selected = createMemo(() => draft());

  const chooseProvider = (provider: ProviderRecord) => {
    setSelectedId(provider.id);
    setDraft(cloneProvider(provider));
    setSaved(false);
    setSaveError(undefined);
  };

  const addProvider = () => {
    setSelectedId(undefined);
    setDraft({
      id: "",
      identifier: "",
      keychainReference: "os-keychain://locus/",
      verificationAt: null,
      verificationModelCount: null,
      verificationStatus: null,
      verificationExpiresAt: null,
      authenticationMethod: "api-key",
      baseUrl: null,
      models: [],
    });
    setSaved(false);
    setSaveError(undefined);
  };

  const updateDraft = (patch: Partial<ProviderDraft>) => {
    setDraft((current) => (current ? { ...current, ...patch } : current));
    setSaved(false);
  };

  const persistProvider = async () => {
    const current = selected();
    if (!current?.identifier.trim() || !current.keychainReference.trim()) {
      setSaveError("Provider identifier and keychain reference are required.");
      return;
    }
    setSaveError(undefined);
    const result = await saveProvider({
      id: current.id || undefined,
      identifier: current.identifier,
      keychainReference: current.keychainReference,
      authenticationMethod: current.authenticationMethod,
      baseUrl: current.baseUrl ?? undefined,
    });
    if (result.status !== "ready") {
      setSaveError(
        result.status === "failed"
          ? result.error.message
          : "Provider was not saved.",
      );
      return;
    }
    setProviders({
      status: "ready",
      data: [
        ...rows().filter((provider) => provider.id !== result.data.id),
        result.data,
      ].sort((left, right) => left.identifier.localeCompare(right.identifier)),
    });
    setSelectedId(result.data.id);
    setDraft(cloneProvider(result.data));
    setSaved(true);
  };

  const persistModels = async (models: ProviderModel[]) => {
    const current = selected();
    if (!current?.id) return;
    updateDraft({ models });
    const result = await saveProviderModels(current.id, models);
    if (result.status === "failed") setSaveError(result.error.message);
    else if (result.status === "ready") setSaved(true);
  };

  const replaceSecret = async () => {
    const current = selected();
    if (!current?.id || !replacementSecret()) return;
    setSaveError(undefined);
    const result = await replaceProviderSecret(current.id, replacementSecret());
    if (result.status === "ready") {
      setReplacementSecret("");
      setSaved(true);
    } else if (result.status === "failed") {
      setSaveError(result.error.message);
    }
  };

  const verification = (provider: ProviderRecord) => {
    if (provider.verificationStatus === "failed") return "verification failed";
    if (provider.verificationStatus !== "verified") return "not verified";
    return `${provider.verificationModelCount ?? 0} models listed`;
  };

  return (
    <div class="ws-providers" data-testid="workshop-providers">
      <aside class="ws-provider-list">
        <header>
          <h1>Providers</h1>
          <p>Credentials and the model catalogue stored by Locus.</p>
          <Button
            variant="primary"
            onClick={addProvider}
            data-testid="provider-add"
          >
            Add provider
          </Button>
        </header>
        <Show when={providers().status === "ready"}>
          <For each={rows()}>
            {(provider) => (
              <button
                type="button"
                class="ws-provider-row"
                aria-selected={selectedId() === provider.id}
                onClick={() => chooseProvider(provider)}
              >
                <i data-state={providerStatus(provider)} />
                {provider.identifier}
                <small>{verification(provider)}</small>
              </button>
            )}
          </For>
        </Show>
        <Show when={providers().status === "empty"}>
          <p data-testid="workshop-providers-empty">
            No model providers are configured.
          </p>
        </Show>
        <Show when={providers().status === "loading"}>
          <p data-testid="workshop-providers-loading">Loading providers…</p>
        </Show>
        <Show when={providers().status === "failed"}>
          <InlineError
            cause={loadError()}
            next="Check the Locus store connection."
          />
        </Show>
        <footer data-testid="provider-keychain-note">
          Secrets live in the OS keychain. Locus stores only the reference and
          model catalogue.
        </footer>
      </aside>

      <Show
        when={selected()}
        fallback={
          <main class="ws-provider-main">
            <p>
              Select a provider or add one to edit its persisted configuration.
            </p>
          </main>
        }
      >
        {(provider) => (
          <main class="ws-provider-main">
            <header class="ws-fixture-head">
              <div>
                <h1>{provider().identifier || "New provider"}</h1>
                <p class="mono">provider/{provider().id || "new"}</p>
              </div>
              <div class="ws-actions">
                <Button
                  variant="secondary"
                  disabled
                  title="Verification is host-side and has no command yet."
                  data-testid="provider-test-connection"
                >
                  Test connection
                </Button>
                <Button
                  variant="primary"
                  onClick={persistProvider}
                  data-testid="provider-save"
                >
                  Save
                </Button>
              </div>
            </header>
            <Show when={saveError()}>
              <InlineError
                cause={saveError()!}
                next="Correct the provider configuration and save again."
              />
            </Show>
            <Show when={saved()}>
              <p data-testid="provider-saved">Saved to the Locus store.</p>
            </Show>

            <section>
              <h2>Authentication</h2>
              <div class="ws-settings-card">
                <div>
                  <span>method</span>
                  <Segmented
                    label="Authentication method"
                    value={provider().authenticationMethod}
                    options={AUTHENTICATION_METHODS}
                    onChange={(value) =>
                      updateDraft({
                        authenticationMethod:
                          value as ProviderDraft["authenticationMethod"],
                      })
                    }
                  />
                </div>
                <div>
                  <span>keychain reference</span>
                  <Input
                    mono
                    value={provider().keychainReference}
                    onInput={(event) =>
                      updateDraft({
                        keychainReference: event.currentTarget.value,
                      })
                    }
                    aria-label="Keychain reference"
                  />
                  <small>secret never crosses the UI</small>
                </div>
                <div>
                  <span>replace credential</span>
                  <Input
                    type="password"
                    value={replacementSecret()}
                    onInput={(event) =>
                      setReplacementSecret(event.currentTarget.value)
                    }
                    aria-label="Replacement provider secret"
                    placeholder="enter in host keychain flow"
                  />
                  <Button
                    variant="secondary"
                    disabled={!provider().id || !replacementSecret()}
                    onClick={replaceSecret}
                    data-testid="provider-replace-secret"
                  >
                    Replace
                  </Button>
                </div>
                <div>
                  <span>base_url</span>
                  <Input
                    mono
                    value={provider().baseUrl ?? ""}
                    onInput={(event) =>
                      updateDraft({
                        baseUrl: event.currentTarget.value || null,
                      })
                    }
                    aria-label="Provider base URL"
                    placeholder="optional override"
                  />
                </div>
                <p data-testid="provider-verification">
                  {verification(provider())}
                </p>
              </div>
            </section>

            <section>
              <h2>Preferred models</h2>
              <Input
                value={modelSearch()}
                onInput={(event) => setModelSearch(event.currentTarget.value)}
                aria-label="Search provider models"
                placeholder="search catalogue"
                data-testid="provider-model-search"
              />
              <div class="ws-model-table">
                <div class="ws-model-head">
                  <span>Model</span>
                  <span>Alias</span>
                  <span>In selector</span>
                </div>
                <For
                  each={provider().models.filter(
                    (model) =>
                      !modelSearch() ||
                      `${model.modelId} ${model.alias ?? ""}`
                        .toLowerCase()
                        .includes(modelSearch().toLowerCase()),
                  )}
                >
                  {(model) => (
                    <div class="ws-model-row">
                      <code>{model.modelId}</code>
                      <Input
                        value={model.alias ?? ""}
                        aria-label={`${model.modelId} alias`}
                        onInput={(event) =>
                          persistModels(
                            provider().models.map((candidate) =>
                              candidate.modelId === model.modelId
                                ? {
                                    ...candidate,
                                    alias: event.currentTarget.value || null,
                                  }
                                : candidate,
                            ),
                          )
                        }
                      />
                      <label>
                        <input
                          type="checkbox"
                          checked={model.selectorIncluded}
                          onChange={(event) =>
                            persistModels(
                              provider().models.map((candidate) =>
                                candidate.modelId === model.modelId
                                  ? {
                                      ...candidate,
                                      selectorIncluded:
                                        event.currentTarget.checked,
                                    }
                                  : candidate,
                              ),
                            )
                          }
                        />
                        include
                      </label>
                    </div>
                  )}
                </For>
                <Show when={provider().models.length === 0}>
                  <p>No curated models are stored for this provider.</p>
                </Show>
              </div>
            </section>
          </main>
        )}
      </Show>

      <aside class="ws-provider-preview">
        <h2>Security boundary</h2>
        <div>
          <strong>OS keychain</strong>
          <span>
            Only a reference is persisted. Credentials are never returned by
            IPC.
          </span>
        </div>
        <h2>Verification</h2>
        <Tag variant="neutral">
          {liveMode ? "host controlled" : "demo provider"}
        </Tag>
      </aside>
    </div>
  );
}
