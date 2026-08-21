import { Icon } from '../../ui/Icon'
import type { InboxItem, InboxKind } from '../../data/inbox'

/** Each kind gets its own glyph, so the list is readable without reading it. */
const KIND_ICON: Record<InboxKind, { name: string; weight: 'regular' | 'fill'; color: string }> = {
  gate: { name: 'seal-check', weight: 'fill', color: 'var(--ac)' },
  ask: { name: 'question', weight: 'regular', color: 'var(--mu)' },
  guardrail: { name: 'warning-octagon', weight: 'fill', color: 'var(--bad)' },
}

export interface InboxCardProps {
  item: InboxItem
  selected: boolean
  onSelect: () => void
}

const age = (minutes: number) => (minutes < 60 ? `${minutes}m` : `${Math.floor(minutes / 60)}h`)

export function InboxCard(props: InboxCardProps) {
  const icon = () => KIND_ICON[props.item.kind]

  return (
    <button
      type="button"
      class="inbox-card"
      data-testid={`inbox-card-${props.item.id}`}
      data-kind={props.item.kind}
      aria-selected={props.selected ? 'true' : 'false'}
      onClick={props.onSelect}
    >
      <div class="inbox-card-head">
        <Icon
          name={icon().name}
          weight={icon().weight}
          size={13}
          style={{ color: icon().color, 'flex-shrink': 0 }}
        />
        <span class="inbox-card-title">{props.item.title}</span>
        <span class="inbox-card-age" data-testid="inbox-card-age">
          {age(props.item.ageMinutes)}
        </span>
      </div>
      <div class="inbox-card-sub" data-testid="inbox-card-sub">
        {props.item.project} · {props.item.agent} · <span class="mono">{props.item.branch}</span>
      </div>
    </button>
  )
}
