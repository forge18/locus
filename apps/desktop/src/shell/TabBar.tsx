import { For, Show } from "solid-js";
import { Icon } from "../ui/Icon";
import { CATEGORY_LABELS, activeTabFor, categoryOf, tabsFor } from "../nav";
import type { View } from "../nav";

export interface TabBarProps {
  view: View;
  onNavigate: (view: View) => void;
  /** The mono locator for the current view, without the scheme. */
  locator: string;
  onDetach?: () => void;
}

/** Only the current category's tabs. single-view categories have none. */
export function TabBar(props: TabBarProps) {
  const tabs = () => tabsFor(categoryOf(props.view));
  const lit = () => activeTabFor(props.view);
  const moveFocus = (event: KeyboardEvent) => {
    if (!(event.key === "ArrowLeft" || event.key === "ArrowRight" || event.key === "Home" || event.key === "End")) return;
    const target = event.currentTarget;
    if (!(target instanceof HTMLButtonElement)) return;
    const buttons = Array.from(
      target.parentElement?.querySelectorAll<HTMLButtonElement>("[role=tab]") ?? [],
    );
    if (!buttons.length) return;
    event.preventDefault();
    const current = buttons.indexOf(target);
    const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1 : (current + (event.key === "ArrowRight" ? 1 : -1) + buttons.length) % buttons.length;
    buttons[next]?.focus();
  };

  return (
    <div class="tabbar" data-testid="tabbar">
      <span class="tabbar-category" data-testid="tabbar-category">
        {CATEGORY_LABELS[categoryOf(props.view)]}
      </span>
      <div class="tabs-list" data-testid="tabbar-tabs" role="tablist" aria-label={`${CATEGORY_LABELS[categoryOf(props.view)]} views`}>
        <For each={tabs()}>
          {(tab) => (
            <button
              type="button"
              class="tab"
              role="tab"
              data-testid={`tab-${tab.view}`}
              data-selected={lit() === tab.view ? "" : undefined}
              aria-selected={lit() === tab.view}
              tabIndex={lit() === tab.view ? 0 : -1}
              onKeyDown={moveFocus}
              onClick={() => props.onNavigate(tab.view)}
            >
              {tab.label}
            </button>
          )}
        </For>
      </div>
      <div class="tabbar-locator" data-testid="tabbar-locator">
        <span>{props.locator}</span>
        <Show when={props.onDetach}>
          <button
            type="button"
            class="btn btn-ghost"
            aria-label="Detach"
            onClick={props.onDetach}
            style={{ padding: "0" }}
          >
            <Icon name="arrows-out-simple" size={12} />
          </button>
        </Show>
      </div>
    </div>
  );
}
