import { For, Show, createSignal, onMount } from "solid-js";
import { dataProvider } from "../../data/provider";
import {
  fetchGuardrails,
  PRIORITY_METHODS,
  saveGuardrails,
  SETTINGS_NAVIGATION,
  type GuardrailSettingsPayload,
  useGuardrails,
} from "../../data/guardrails";
import type { GuardrailControl, GuardrailSection } from "../../data/guardrails";
import { AppearanceSelector } from "./AppearanceSelector";
import { Button } from "../../ui/Button";
import { AvatarStylePicker } from "./AvatarStylePicker";
import { notify } from "../../ui/Toast";

function stepValue(value: string, delta: number): string {
  const match = /^(\d+)([kKmM]?)$/.exec(value);
  if (!match) return value;
  const suffix = match[2].toLowerCase();
  return `${Math.max(1, Number(match[1]) + delta)}${suffix}`;
}

function Stepper(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "stepper" }>;
  onChange: (value: string) => void;
}) {
  return (
    <div class="settings-stepper" data-testid={`settings-stepper-${props.id}`}>
      <button
        type="button"
        aria-label={`Decrease ${props.id}`}
        onClick={() => props.onChange(stepValue(props.control.value, -1))}
      >
        −
      </button>
      <span class="mono" data-testid={`settings-value-${props.id}`}>
        {props.control.value}
      </span>
      <button
        type="button"
        aria-label={`Increase ${props.id}`}
        onClick={() => props.onChange(stepValue(props.control.value, 1))}
      >
        +
      </button>
    </div>
  );
}

function Toggle(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "toggle" }>;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      type="button"
      class="settings-toggle"
      data-testid={`settings-toggle-${props.id}`}
      data-on={props.control.value ? "true" : "false"}
      aria-checked={props.control.value}
      role="switch"
      aria-label={props.id}
      onClick={() => props.onChange(!props.control.value)}
    >
      <span />
    </button>
  );
}

const SELECT_OPTIONS: Record<string, string[]> = {
  "priority-method": [
    "plan order",
    "manual",
    "unblocks most",
    "shortest first",
  ],
  "tie-break": ["longest waiting"],
  "network-tier": ["open", "allowlist", "none"],
};

function Select(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "select" }>;
  onChange: (value: string) => void;
}) {
  const options = () => SELECT_OPTIONS[props.id] ?? [props.control.value];
  return (
    <button
      type="button"
      class="settings-select"
      data-testid={`settings-value-${props.id}`}
      onClick={() => {
        const values = options();
        const index = values.indexOf(props.control.value);
        props.onChange(values[(index + 1) % values.length]);
      }}
    >
      {props.control.value}
      <span aria-hidden="true">⌄</span>
    </button>
  );
}

function Control(props: {
  id: string;
  control: GuardrailControl;
  onChange: (value: string | boolean) => void;
}) {
  switch (props.control.kind) {
    case "stepper":
      return (
        <Stepper
          id={props.id}
          control={props.control}
          onChange={(value) => props.onChange(value)}
        />
      );
    case "toggle":
      return (
        <Toggle
          id={props.id}
          control={props.control}
          onChange={(value) => props.onChange(value)}
        />
      );
    case "select":
      return (
        <Select
          id={props.id}
          control={props.control}
          onChange={(value) => props.onChange(value)}
        />
      );
  }
}

/** Guardrail defaults are live in Tauri and fixture-backed only in the demo/test host. */
export function GuardrailsView() {
  const liveMode = dataProvider().kind === "live";
  const shipped = useGuardrails();
  const [sections, setSections] = createSignal<readonly GuardrailSection[]>(
    liveMode ? [] : shipped,
  );
  const [loading, setLoading] = createSignal(liveMode);
  const [loadError, setLoadError] = createSignal<string>();
  const [saved, setSaved] = createSignal(false);

  const refresh = async () => {
    if (!liveMode) return;
    setLoading(true);
    setLoadError(undefined);
    const envelope = await fetchGuardrails();
    if (envelope.status === "ready") setSections(envelope.data);
    else if (envelope.status === "empty") setSections([]);
    else if (envelope.status === "failed") setLoadError(envelope.error.message);
    setLoading(false);
  };

  onMount(() => {
    void refresh();
  });

  const updateControl = (id: string, value: string | boolean) => {
    setSections((current) =>
      current.map((section) => ({
        ...section,
        settings: section.settings.map((setting) => {
          if (setting.id !== id) return setting;
          const control =
            setting.control.kind === "toggle"
              ? { kind: "toggle" as const, value: Boolean(value) }
              : setting.control.kind === "stepper"
                ? { kind: "stepper" as const, value: String(value) }
                : { kind: "select" as const, value: String(value) };
          return { ...setting, control };
        }),
      })),
    );
    setSaved(false);
  };

  const controlValue = (id: string): string | boolean | undefined => {
    for (const section of sections()) {
      const setting = section.settings.find((candidate) => candidate.id === id);
      if (setting) return setting.control.value;
    }
    return undefined;
  };
  const numberValue = (id: string): number | null => {
    const value = controlValue(id);
    if (typeof value !== "string" || value === "unlimited") return null;
    const match = /^(\\d+)([kKmM]?)$/.exec(value);
    if (!match) return null;
    const multiplier =
      match[2].toLowerCase() === "k"
        ? 1000
        : match[2].toLowerCase() === "m"
          ? 1_000_000
          : 1;
    return Number(match[1]) * multiplier;
  };
  const booleanValue = (id: string) => controlValue(id) === true;
  const stringValue = (id: string) => String(controlValue(id) ?? "");
  const settingsPayload = (): GuardrailSettingsPayload => ({
    maxIterations: numberValue("max-iterations") ?? 1,
    tokenBudget: numberValue("token-budget"),
    stuckIterations: numberValue("stuck-detection") ?? 1,
    killAndReassign: booleanValue("kill-reassign"),
    globalParallelism: numberValue("max-parallel-agents") ?? 1,
    perProjectParallelism: numberValue("max-per-project") ?? 1,
    priorityMethod: stringValue("priority-method"),
    tieBreak: stringValue("tie-break"),
    changeLinesCeiling: numberValue("lines-changed"),
    changeFilesCeiling: numberValue("files-touched"),
    networkTier: stringValue("network-tier"),
    blockSystemChanges: booleanValue("block-system-changes"),
    autopilot: booleanValue("autopilot"),
  });

  const saveDefaults = async () => {
    if (!liveMode) {
      setSaved(true);
      return;
    }
    const envelope = await saveGuardrails(settingsPayload());
    if (envelope.status === "ready") {
      setSections(envelope.data);
      setSaved(true);
      notify({ title: "Guardrail defaults saved" });
    } else if (envelope.status === "failed") {
      notify({
        title: "Guardrail save failed",
        description: envelope.error.message,
        type: "error",
      });
    }
  };
  const resetDefaults = () => {
    if (liveMode) void refresh();
    else setSections(shipped);
    setSaved(false);
  };

  return (
    <div class="settings" data-testid="settings">
      <aside class="settings-rail" data-testid="settings-rail">
        <h1>Settings</h1>
        <nav aria-label="Settings sections">
          <For each={SETTINGS_NAVIGATION}>
            {(item) => (
              <button
                type="button"
                class="settings-nav-item"
                classList={{ "settings-nav-active": item === "Guardrails" }}
                data-testid={`settings-nav-${item.toLowerCase().replace(/[^a-z]+/g, "-")}`}
                aria-current={item === "Guardrails" ? "page" : undefined}
              >
                {item}
              </button>
            )}
          </For>
        </nav>
        <p data-testid="settings-install-note">
          Settings are per install. Anything scoped to a project lives in that
          project’s base context instead.
        </p>
      </aside>

      <main class="settings-body">
        <div class="settings-content">
          <header class="settings-head">
            <h2>Guardrails</h2>
            <p>
              Defaults for every new run. A run can be given tighter limits than
              these; it can never be given looser ones without an explicit
              override that is recorded on the run.
            </p>
          </header>

          <Show when={loading()}>
            <p data-testid="guardrails-loading">Loading guardrail defaults…</p>
          </Show>
          <Show when={loadError()}>
            <p data-testid="guardrails-error">{loadError()}</p>
          </Show>
          <Show
            when={
              liveMode && !loading() && !loadError() && sections().length === 0
            }
          >
            <p data-testid="guardrails-empty">
              No guardrail settings are available.
            </p>
          </Show>
          <Show
            when={
              !liveMode || (!loading() && !loadError() && sections().length > 0)
            }
          >
            <Show when={!liveMode}>
              <AppearanceSelector />
              <AvatarStylePicker />
            </Show>
            <For each={sections()}>
              {(section) => (
                <section
                  class="settings-section"
                  data-testid={
                    section.id === "parallelism"
                      ? "parallelism-controls"
                      : `settings-section-${section.id}`
                  }
                >
                  <h3>{section.label}</h3>
                  <For each={section.settings}>
                    {(setting) => (
                      <div
                        class="settings-row"
                        data-testid={`settings-row-${setting.id}`}
                      >
                        <div class="settings-copy">
                          <span>{setting.label}</span>
                          <p
                            data-testid={
                              setting.id === "preempt"
                                ? "settings-preempt-note"
                                : undefined
                            }
                          >
                            {setting.description}
                          </p>
                          <Show when={setting.id === "priority-method"}>
                            <div
                              class="settings-priority-options"
                              data-testid="settings-priority-method"
                            >
                              <For each={PRIORITY_METHODS}>
                                {([method, note]) => (
                                  <div>
                                    <span class="mono">{method}</span> — {note}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                        <div class="settings-control">
                          <Control
                            id={setting.id}
                            control={setting.control}
                            onChange={(value) =>
                              updateControl(setting.id, value)
                            }
                          />
                        </div>
                      </div>
                    )}
                  </For>
                </section>
              )}
            </For>
            <footer
              class="settings-guardrails-footer"
              data-testid="guardrails-save-footer"
            >
              <Show when={saved()}>
                <span class="settings-saved">
                  Saved — applies to runs started after saving. Nothing in
                  flight is retuned underneath itself.
                </span>
              </Show>
              <Button variant="ghost" onClick={resetDefaults}>
                Reset to shipped values
              </Button>
              <Button onClick={() => void saveDefaults()}>Save defaults</Button>
            </footer>
          </Show>
        </div>
      </main>
    </div>
  );
}

export default GuardrailsView;
