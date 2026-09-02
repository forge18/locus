import { For, Show, createSignal, onMount } from "solid-js";
import { Button } from "../../ui/Button";
import { Combobox } from "../../ui/Combobox";
import { Icon } from "../../ui/Icon";
import { Input } from "../../ui/Input";
import {
  EXTENSION_LABELS,
  fetchHarnesses,
  useExtensionTypes,
  useHarnessSummary,
  useHarnesses,
} from "../../data/harnesses";
import { dataProvider } from "../../data/provider";
import type { Envelope } from "../../data/envelope";
import type { ExtensionType, HarnessEntry } from "../../data/harnesses";
import {
  TIERS,
  fallbackMarker,
  resolveTier,
  useHarnessTierGrid,
} from "../../data/settings";
import type { ModelTier } from "../../types/core";

/** At four or more, the downgrades are the story rather than a footnote. */
const HEAVY = 4;

export const HEADER_NOTE =
  "Mechanism lives in the file; policy lives here. Every harness has every capability — only the mechanism differs.";

/** The count comes from the registry too — a literal here goes stale on the next one. */
export const tuiNote = (count: number) =>
  `tui = false is required on all ${count}; a harness claiming true is refused at registration.`;

/**
 * Every figure on this screen is computed from harnesses/*.toml. Registering a
 * thirteenth harness moves all of them without an edit here — which is the whole
 * argument for the registry being a file rather than a table in the source.
 */
export function HarnessesView() {
  const liveMode = dataProvider().kind === "live";
  const fixtureHarnesses = useHarnesses();
  const [registry, setRegistry] = createSignal<Envelope<HarnessEntry[]>>(
    liveMode
      ? { status: "loading" }
      : fixtureHarnesses.length > 0
        ? { status: "ready", data: [...fixtureHarnesses] }
        : { status: "empty" },
  );
  const fixtureSummary = useHarnessSummary();
  const fixtureTypes = useExtensionTypes();
  const tierGrid = useHarnessTierGrid();

  onMount(() => {
    if (liveMode) void fetchHarnesses().then(setRegistry);
  });

  const harnesses = (): readonly HarnessEntry[] => {
    const state = registry();
    return state.status === "ready" ? state.data : [];
  };
  const types = (): readonly ExtensionType[] =>
    liveMode
      ? (Array.from(
          new Set(
            harnesses().flatMap((harness) =>
              harness.extensions.map((extension) => extension.type),
            ),
          ),
        ) as ExtensionType[])
      : fixtureTypes;
  const summary = () => {
    if (!liveMode) return fixtureSummary;
    const entries = harnesses().reduce(
      (total, harness) => total + harness.extensions.length,
      0,
    );
    const downgrades = harnesses().reduce(
      (total, harness) =>
        total +
        harness.extensions.filter((extension) => extension.weakerThanNative)
          .length,
      0,
    );
    return {
      harnesses: harnesses().length,
      entries,
      downgrades,
    };
  };

  return (
    <div class="harnesses" data-testid="harnesses">
      <Show when={liveMode && registry().status === "loading"}>
        <p data-testid="harnesses-loading">Loading the harness registry…</p>
      </Show>
      <Show when={liveMode && registry().status === "failed"}>
        <p data-testid="harnesses-error">
          {(() => {
            const state = registry();
            return state.status === "failed" ? state.error.message : "";
          })()}
        </p>
      </Show>
      <Show when={liveMode && registry().status === "empty"}>
        <p data-testid="harnesses-empty">No harnesses are registered.</p>
      </Show>
      <header class="ws-head" data-testid="harnesses-head">
        <span class="ws-title" data-testid="harnesses-title">
          Registered harnesses{" "}
          <span class="mono" data-testid="harnesses-count">
            {summary().harnesses}
          </span>
        </span>
        <span class="ws-note" data-testid="harnesses-note">
          {HEADER_NOTE}
        </span>
        <div class="ws-actions">
          <div class="hn-legend" data-testid="harnesses-legend">
            <span class="hn-legend-native">
              <i />
              native
            </span>
            <span class="hn-legend-downgraded">
              <i />
              downgraded — each names its loss
            </span>
          </div>
          <Button variant="primary" data-testid="harnesses-register">
            <Icon name="plus" size={11} />
            Register a harness
          </Button>
        </div>
      </header>

      <div class="hn-grid" data-testid="harnesses-grid">
        <For each={harnesses()}>
          {(harness) => {
            const downgrades = harness.extensions.filter(
              (e) => e.weakerThanNative,
            ).length;
            const heavy = downgrades >= HEAVY;
            const tierSettings = tierGrid.find(
              (row) => row.name === harness.name,
            ) ?? {
              name: harness.name,
              models: null,
              tiers: TIERS.map((tier) => ({
                harness: harness.name,
                tier,
                model: null,
              })),
            };
            return (
              <article
                class={["hn-card", heavy ? "hn-card-heavy" : ""]
                  .filter(Boolean)
                  .join(" ")}
                data-testid={`hn-card-${harness.name}`}
                data-heavy={heavy ? "true" : undefined}
              >
                <div class="hn-card-head">
                  <span class="hn-name">{harness.name}</span>
                  <span class="hn-id">{harness.binary}</span>
                  <span
                    class={`hn-badge hn-badge-${harness.badge.variant}`}
                    data-testid={`hn-badge-${harness.name}`}
                    data-variant={harness.badge.variant}
                  >
                    {harness.badge.label}
                  </span>
                </div>

                <span
                  class="hn-injection"
                  data-testid={`hn-injection-${harness.name}`}
                >
                  injection: {harness.injection}
                </span>

                <div class="hn-tiers" data-testid={`hn-tiers-${harness.name}`}>
                  <For each={TIERS}>
                    {(tier: ModelTier) => {
                      const setting = tierSettings.tiers.find(
                        (entry) => entry.tier === tier,
                      )!;
                      const resolved = resolveTier(harness.name, tier);
                      const own = resolved.fellBackTo === null;
                      const options = tierSettings.models?.map((model) => ({
                        value: model,
                        label: model,
                      }));
                      const selected =
                        options?.find(
                          (option) => option.value === setting.model,
                        ) ?? null;
                      return (
                        <div
                          class={[
                            "hn-tier",
                            tier === "high" ? "hn-tier-high" : "",
                          ]
                            .filter(Boolean)
                            .join(" ")}
                          data-testid={`hn-tier-${harness.name}-${tier}`}
                          data-fallback={own ? undefined : resolved.fellBackTo!}
                        >
                          <span class="hn-tier-label">
                            {tier === "medium" ? "med" : tier}
                          </span>
                          <Show
                            when={own}
                            fallback={
                              <span
                                class="hn-tier-fallback"
                                data-testid={`hn-fallback-${harness.name}-${tier}`}
                              >
                                {fallbackMarker(resolved.fellBackTo!)}
                              </span>
                            }
                          >
                            <span class="hn-tier-value hn-tier-preview">
                              {setting.model}
                            </span>
                          </Show>
                          <div
                            class="hn-tier-editor"
                            data-testid={`settings-tier-${harness.name}-${tier}`}
                            data-editor={options ? "combobox" : "free-text"}
                          >
                            <Show
                              when={options}
                              fallback={
                                <Input
                                  class="hn-tier-input"
                                  mono
                                  readOnly
                                  value={setting.model ?? ""}
                                  placeholder="harness default"
                                  aria-label={`${harness.name} ${tier} model`}
                                />
                              }
                            >
                              <Combobox
                                options={options!}
                                value={selected}
                                onChange={() => undefined}
                                placeholder="harness default"
                                label={`${harness.name} ${tier} model`}
                              />
                            </Show>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                </div>

                <div class="hn-bar" data-testid={`hn-bar-${harness.name}`}>
                  <For each={types()}>
                    {(type) => {
                      const entry = harness.extensions.find(
                        (e) => e.type === type,
                      )!;
                      const native = entry.weakerThanNative === null;
                      return (
                        <span
                          class={`hn-seg hn-seg-${native ? "native" : "downgraded"}`}
                          data-testid={`hn-seg-${harness.name}-${type}`}
                          data-native={native ? "true" : "false"}
                          title={
                            native
                              ? `${EXTENSION_LABELS[type]}: native`
                              : `${EXTENSION_LABELS[type]}: ${entry.weakerThanNative}`
                          }
                        />
                      );
                    }}
                  </For>
                </div>

                <div class="hn-foot">
                  <span data-testid={`hn-extension-count-${harness.name}`}>
                    {types().length} extensions
                  </span>
                  <span
                    class={["hn-downgrades", heavy ? "hn-downgrades-bad" : ""]
                      .filter(Boolean)
                      .join(" ")}
                    data-testid={`hn-downgrades-${harness.name}`}
                  >
                    {downgrades === 0
                      ? "all native"
                      : `${downgrades} downgraded`}
                  </span>
                </div>
              </article>
            );
          }}
        </For>
      </div>

      <footer class="harnesses-foot" data-testid="harnesses-foot">
        <span data-testid="harnesses-downgrade-line">
          {summary().downgrades} of {summary().entries} entries are downgrades —
          the honest measure of how uneven the field is.
        </span>{" "}
        <span class="mono" data-testid="harnesses-tui-note">
          {tuiNote(summary().harnesses)}
        </span>
      </footer>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default HarnessesView;
