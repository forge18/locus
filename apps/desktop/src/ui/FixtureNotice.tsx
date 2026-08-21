import { Icon } from './Icon'

export interface FixtureNoticeProps {
  /** What is not wired yet, named plainly. */
  surface: string
  /** The Tauri command that will replace the fixture. */
  command: string
}

/**
 * Marks a screen whose backend does not exist yet. A screen showing invented data
 * without saying so is worse than an empty one — the reader cannot tell which
 * numbers to trust, so they stop trusting all of them.
 */
export function FixtureNotice(props: FixtureNoticeProps) {
  return (
    <div
      class="fixture-notice"
      role="status"
      data-testid="fixture-notice"
      style={{
        display: 'flex',
        'align-items': 'center',
        gap: 'var(--g-3)',
        padding: '3px var(--g-4)',
        'border-radius': 'var(--r-sm)',
        background: 'var(--sf)',
        'box-shadow': 'inset 0 0 0 1px var(--line2)',
        color: 'var(--mu)',
      }}
    >
      <Icon name="info" size={11} style={{ color: 'var(--ac)' }} />
      <span class="t-meta" data-testid="fixture-notice-surface">
        {props.surface} is fixture data — no backend yet
      </span>
      <span class="t-meta mono" style={{ color: 'var(--mu2)' }} data-testid="fixture-notice-command">
        {props.command}
      </span>
    </div>
  )
}
