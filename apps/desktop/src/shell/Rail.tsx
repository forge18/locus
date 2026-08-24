import { For, Show } from 'solid-js'
import { Icon } from '../ui/Icon'
import { RAIL_ITEMS, categoryOf } from '../nav'
import type { Category, View } from '../nav'

export interface RailProps {
  view: View
  onNavigate: (view: View) => void
  /** Hidden at zero. Silence is the default, and it should be legible from anywhere. */
  inboxCount: number
}

/**
 * Seven items, and the list is closed. The rail lights by **category**, so
 * drilling into agent definitions keeps Workshop lit rather than adding an eighth.
 */
export function Rail(props: RailProps) {
  const active = (): Category => categoryOf(props.view)

  return (
    <nav class="rail" data-testid="rail" aria-label="Categories">
      <For each={RAIL_ITEMS}>
        {(item) => (
          <button
            type="button"
            class="rail-item"
            data-testid={`rail-${item.category}`}
            data-category={item.category}
            aria-current={active() === item.category ? 'true' : undefined}
            onClick={() => props.onNavigate(item.firstView)}
          >
            <Icon name={item.icon} size={19} />
            <span class="rail-item-label">{item.label}</span>
            <Show when={item.category === 'pill' && props.inboxCount > 0}>
              <span class="rail-badge" data-testid="inbox-badge">
                {props.inboxCount}
              </span>
            </Show>
          </button>
        )}
      </For>
      <div class="rail-foot" data-testid="rail-foot">
        <Icon name="git-branch" size={13} />
        <Icon name="user-circle" size={13} />
      </div>
    </nav>
  )
}
