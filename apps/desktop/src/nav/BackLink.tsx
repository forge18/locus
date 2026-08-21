import { Show } from 'solid-js'
import { Icon } from '../ui/Icon'
import { categoryOf } from './views'
import { drilldownParent, tabsFor } from './tabs'
import type { NavStore } from './store'

export interface BackLinkProps {
  nav: NavStore
}

/**
 * Renders only on a drill-down, and only in the sidebar of the screen that is one.
 * A drill-down is not a category, so the way out of it is a link back to where it
 * was entered from rather than another rail item.
 */
export function BackLink(props: BackLinkProps) {
  const parent = () => drilldownParent(props.nav.view())
  /** Label it with the tab it goes back to, so the link and the lit tab agree. */
  const label = () => tabsFor(categoryOf(parent()!)).find((t) => t.view === parent())?.label ?? ''

  return (
    <Show when={parent()}>
      <button
        type="button"
        class="btn btn-ghost"
        data-testid="drilldown-back"
        onClick={() => props.nav.go(parent()!)}
        style={{ gap: 'var(--g-2)', 'font-size': '11px' }}
      >
        <Icon name="arrow-left" size={11} />
        {label()}
      </button>
    </Show>
  )
}
