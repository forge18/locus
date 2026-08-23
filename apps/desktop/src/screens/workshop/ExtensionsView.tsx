import { createSignal, For, onMount, Show } from "solid-js";
import { Button } from "../../ui/Button";
import { Icon } from "../../ui/Icon";
import { InlineError } from "../../ui/InlineError";
import { Input } from "../../ui/Input";
import { Tag } from "../../ui/Tag";
import {
  CACHE_READ_RATE,
  DETERMINISM_NOTE,
  ENTRY_TYPE,
  HEADER_NOTE,
  HEADER_TITLE,
  NEW_LABEL,
  SEARCH_PLACEHOLDER,
  fetchLinterCountFromCore,
  useRecentlyEdited,
  useTypeCards,
} from "../../data/extensions";
import { useExtensionCounts, useHarnessSummary } from "../../data/harnesses";
import type { View } from "../../nav";

export interface ExtensionsViewProps {
  onNavigate: (view: View) => void;
}

/**
 * The one surface. Every native/downgraded figure here is computed from
 * harnesses/*.toml, so registry changes cannot leave the interface stale.
 */
export function ExtensionsView(props: ExtensionsViewProps) {
  const counts = useExtensionCounts();
  const summary = useHarnessSummary();
  const [linterCount, setLinterCount] = createSignal<number>();
  const [loadError, setLoadError] = createSignal<string | null>(null);

  onMount(() => {
    void fetchLinterCountFromCore()
      .then((count) => setLinterCount(count))
      .catch((e) => setLoadError(e instanceof Error ? e.message : String(e)));
  });

  const countFor = (type: string) => counts.find((c) => c.type === type)!;
  const displayedCount = (type: string, fallback: number) =>
    type === "linters" ? (linterCount() ?? fallback) : fallback;

  return (
    <div class="workshop" data-testid="extensions">
      <Show when={loadError()}>
        <div data-testid="extensions-error">
          <InlineError
            cause={loadError()!}
            next="Retry the connection to core, or check the core daemon."
          />
        </div>
      </Show>
      <header class="ws-head" data-testid="extensions-head">
        <span class="ws-title" data-testid="extensions-title">
          {HEADER_TITLE}
        </span>
        <span class="ws-note" data-testid="extensions-note">
          {HEADER_NOTE}
        </span>
        <div class="ws-actions">
          <Input
            data-testid="extensions-search"
            placeholder={SEARCH_PLACEHOLDER}
            style={{ width: "180px" }}
          />
          <Button variant="primary" data-testid="extensions-new">
            <Icon name="plus" size={11} />
            {NEW_LABEL}
          </Button>
        </div>
      </header>

      <div class="type-grid" data-testid="type-grid">
        <For each={useTypeCards()}>
          {(card) => {
            const count = countFor(card.type);
            const dominated = count.downgraded > count.native;
            const entry = card.type === ENTRY_TYPE;
            return (
              <button
                type="button"
                class={["type-card", entry ? "type-card-entry" : ""]
                  .filter(Boolean)
                  .join(" ")}
                data-testid={`type-card-${card.type}`}
                data-entry={entry ? "true" : undefined}
                onClick={() => entry && props.onNavigate("agents")}
              >
                <div class="type-card-head">
                  <Icon name={card.icon} size={12} />
                  {card.type}
                  <Show when={entry}>
                    <span class="type-card-arrow" data-testid="type-card-arrow">
                      <Icon name="arrow-right" size={11} />
                    </span>
                  </Show>
                </div>
                <span
                  class="type-card-count"
                  data-testid={`type-count-${card.type}`}
                >
                  {displayedCount(card.type, card.count)}
                </span>
                <span class="type-card-desc">{card.description}</span>
                <span
                  class={[
                    "type-card-foot",
                    dominated ? "type-card-foot-bad" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  data-testid={`type-native-${card.type}`}
                  data-dominated={dominated ? "true" : undefined}
                >
                  {count.native} native · {count.downgraded} downgraded
                </span>
              </button>
            );
          }}
        </For>
      </div>

      <section class="panel" data-testid="recently-edited">
        <span class="panel-title">Recently edited</span>
        <For each={useRecentlyEdited()}>
          {(entry) => (
            <div class="edited-row" data-testid={`edited-${entry.file}`}>
              <Tag variant="neutral">{entry.type}</Tag>
              <span class="edited-file">{entry.file}</span>
              <span class="edited-summary">{entry.summary}</span>
              <span class="edited-age">{entry.age}</span>
            </div>
          )}
        </For>
      </section>

      <section class="panel materialization" data-testid="materialization">
        <span class="panel-title">Materialization</span>
        <span
          class="ws-note"
          style={{ "max-width": "none" }}
          data-testid="determinism-note"
        >
          {DETERMINISM_NOTE}
        </span>
        <div class="materialization-figures">
          <div
            class="materialization-figure"
            data-testid="materialization-entries"
          >
            <span class="materialization-value">{summary.entries}</span>
            <span class="materialization-label">entries</span>
          </div>
          <div
            class="materialization-figure"
            data-testid="materialization-downgrades"
          >
            <span class="materialization-value">{summary.downgrades}</span>
            <span class="materialization-label">downgrades</span>
          </div>
          <div
            class="materialization-figure"
            data-testid="materialization-cache"
          >
            <span class="materialization-value">{CACHE_READ_RATE}</span>
            <span class="materialization-label">cache read</span>
          </div>
        </div>
      </section>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default ExtensionsView;
