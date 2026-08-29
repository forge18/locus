import { For, Show, createSignal } from "solid-js";
import {
  PRIORITY_METHODS,
  SETTINGS_NAVIGATION,
  useGuardrails,
} from "../../data/guardrails";
import type { GuardrailControl } from "../../fixtures/settings-guardrails";
import { AppearanceSelector } from "./AppearanceSelector";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { AvatarStylePicker } from "./AvatarStylePicker";

function Stepper(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "stepper" }>;
}) {
  return (
    <div class="settings-stepper" data-testid={`settings-stepper-${props.id}`}>
      <button type="button" aria-label={`Decrease ${props.id}`}>
        −
      </button>
      <span class="mono" data-testid={`settings-value-${props.id}`}>
        {props.control.value}
      </span>
      <button type="button" aria-label={`Increase ${props.id}`}>
        +
      </button>
    </div>
  );
}

function Toggle(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "toggle" }>;
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
    >
      <span />
    </button>
  );
}

function Select(props: {
  id: string;
  control: Extract<GuardrailControl, { kind: "select" }>;
}) {
  return (
    <button
      type="button"
      class="settings-select"
      data-testid={`settings-value-${props.id}`}
    >
      {props.control.value}
      <span aria-hidden="true">⌄</span>
    </button>
  );
}

function Control(props: { id: string; control: GuardrailControl }) {
  switch (props.control.kind) {
    case "stepper":
      return <Stepper id={props.id} control={props.control} />;
    case "toggle":
      return <Toggle id={props.id} control={props.control} />;
    case "select":
      return <Select id={props.id} control={props.control} />;
  }
}

/** Guardrail defaults fixture; persistence arrives with the dispatch settings command. */
export function GuardrailsView() {
  const shipped = useGuardrails()
  const [sections, setSections] = createSignal(shipped)
  const [saved, setSaved] = createSignal(false)

  const saveDefaults = () => setSaved(true)
  const resetDefaults = () => { setSections(shipped); setSaved(false) }

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
          <FixtureNotice
            surface="Settings · Guardrails"
            command='invoke("guardrail_settings")'
          />
          <header class="settings-head">
            <h2>Guardrails</h2>
            <p>
              Defaults for every new run. A run can be given tighter limits than
              these; it can never be given looser ones without an explicit
              override that is recorded on the run.
            </p>
          </header>

          <AppearanceSelector />
          <AvatarStylePicker />
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
                        <Control id={setting.id} control={setting.control} />
                      </div>
                    </div>
                  )}
                </For>
              </section>
            )}
          </For>
          <footer class="settings-guardrails-footer" data-testid="guardrails-save-footer">
            <Show when={saved()}><span class="settings-saved">Saved — applies to runs started after saving. Nothing in flight is retuned underneath itself.</span></Show>
            <Button variant="ghost" onClick={resetDefaults}>Reset to shipped values</Button>
            <Button onClick={saveDefaults}>Save defaults</Button>
          </footer>
        </div>
      </main>
    </div>
  );
}

export default GuardrailsView;
