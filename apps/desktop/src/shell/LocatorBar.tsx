import { Icon } from '../ui/Icon'

export interface LocatorBarProps {
  /** The path after the scheme, e.g. `tapestry/session/8f21`. */
  path: string
  onOpen?: () => void
}

/**
 * The app's addressing surface, not decoration. Every object has a `locus://` URI
 * and this is where one is read and typed — ⌘K resolves it.
 */
export function LocatorBar(props: LocatorBarProps) {
  return (
    <button class="locator-bar" data-testid="locator-bar" onClick={props.onOpen} type="button">
      <Icon name="magnifying-glass" size={12} style={{ color: 'var(--text-muted)' }} />
      <span class="locator-scheme" data-testid="locator-scheme">
        locus://
      </span>
      <span class="locator-path" data-testid="locator-path">
        {props.path}
      </span>
      <span class="locator-key" data-testid="locator-key">
        ⌘K
      </span>
    </button>
  )
}
