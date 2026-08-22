import { For, Show, createMemo, createSignal } from 'solid-js'
import { InboxCard } from './InboxCard'
import { InboxDetail } from './InboxDetail'
import { EmptyPane } from '../../ui/EmptyPane'
import { Icon } from '../../ui/Icon'
import { useInboxItems, useResolvedToday } from '../../data/inbox'
import type { NavStore } from '../../nav'

export interface InboxViewProps {
  nav: NavStore
}

const RESOLVED_ICON = { gate: 'seal-check', ask: 'question', guardrail: 'warning-octagon' } as const
const age = (minutes: number) => (minutes < 60 ? `${minutes}m` : `${Math.floor(minutes / 60)}h`)

/**
 * The only interruption surface. A decision resolves here; the work it is about
 * opens where that work lives, by locator — this screen never grows a second copy
 * of Plan, Develop or Review.
 */
export function InboxView(props: InboxViewProps) {
  const [resolved, setResolved] = createSignal<string[]>([])
  const [selectedId, setSelectedId] = createSignal<string | null>(null)

  const items = createMemo(() => useInboxItems().filter((i) => !resolved().includes(i.id)))
  const selected = createMemo(
    () => items().find((i) => i.id === selectedId()) ?? items()[0] ?? null,
  )

  /**
   * Resolving is in place: nothing about where you are changes. Named
   * `resolveItem` rather than `resolve` because `resolve` is the navigation
   * resolver, and one word meaning two things here would be a trap.
   */
  const resolveItem = (id: string) => setResolved([...resolved(), id])

  return (
    <div class="inbox" data-testid="inbox">
      <div class="inbox-list" data-testid="inbox-list">
        <div class="inbox-section">
          <span class="inbox-section-title" data-testid="needs-you-title">
            Needs you
          </span>
          <span class="inbox-section-note" data-testid="needs-you-note">
            {items().length} {items().length === 1 ? 'item' : 'items'} · silence is the default
          </span>
        </div>

        <Show
          when={items().length > 0}
          fallback={
            <EmptyPane icon="tray" reason="Nothing needs you" />
          }
        >
          <For each={items()}>
            {(item) => (
              <InboxCard
                item={item}
                selected={selected()?.id === item.id}
                onSelect={() => setSelectedId(item.id)}
              />
            )}
          </For>
        </Show>

        <div class="inbox-section" style={{ 'margin-top': 'var(--g-4)' }}>
          <span
            class="inbox-section-title"
            style={{ color: 'var(--text-muted)' }}
            data-testid="resolved-title"
          >
            Resolved today
          </span>
        </div>
        <div class="inbox-resolved" data-testid="inbox-resolved">
          <For each={useResolvedToday()}>
            {(row) => (
              <div class="inbox-resolved-row" data-testid={`resolved-${row.id}`}>
                <Icon name={RESOLVED_ICON[row.kind]} size={11} />
                <span>{row.title}</span>
                <span style={{ 'margin-left': 'auto', color: 'var(--text-muted)' }}>
                  {age(row.ageMinutes)}
                </span>
              </div>
            )}
          </For>
        </div>
      </div>

      <Show
        when={selected()}
        fallback={
          <EmptyPane reason="Nothing needs you — approve something and it resolves right here." />
        }
      >
        <InboxDetail
          item={selected()!}
          onApprove={() => resolveItem(selected()!.id)}
          onSendBack={() => resolveItem(selected()!.id)}
          onOpenWork={(locator) => props.nav.open(locator)}
        />
      </Show>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default InboxView
