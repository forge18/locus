import { For, Show } from 'solid-js'
import { Icon } from '../ui/Icon'
import { CATEGORY_LABELS, activeTabFor, categoryOf, tabsFor } from '../nav'
import type { View } from '../nav'

export interface TabBarProps {
  view: View
  onNavigate: (view: View) => void
  /** The mono locator for the current view, without the scheme. */
  locator: string
  onDetach?: () => void
}

/** Only the current category's tabs. Plan, Develop and Wiki have none. */
export function TabBar(props: TabBarProps) {
  const tabs = () => tabsFor(categoryOf(props.view))
  const lit = () => activeTabFor(props.view)

  return (
    <div class="tabbar" data-testid="tabbar">
      <span class="tabbar-category" data-testid="tabbar-category">
        {CATEGORY_LABELS[categoryOf(props.view)]}
      </span>
      <div class="tabs-list" data-testid="tabbar-tabs">
        <For each={tabs()}>
          {(tab) => (
            <button
              type="button"
              class="tab"
              data-testid={`tab-${tab.view}`}
              data-selected={lit() === tab.view ? '' : undefined}
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
            style={{ padding: '0' }}
          >
            <Icon name="arrows-out-simple" size={12} />
          </button>
        </Show>
      </div>
    </div>
  )
}
