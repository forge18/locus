import { STRIP_CARDS } from '../fixtures/strip'
import type { StripCard } from '../fixtures/strip'

export type { StripCard, StripKind } from '../fixtures/strip'

/** How badly a card wants a person. Higher goes first. */
function attention(card: StripCard): number {
  if (card.status === 'stuck') return 3
  if (card.status === 'waiting') return 2
  if (card.status === 'idle') return 1
  return 0
}

/**
 * Becomes: invoke("strip_cards") + emit("session_status_changed")
 *
 * Sorted needs-attention first, then activity. Never by project and never
 * alphabetically — either would put the same session in the same place whether or
 * not anything was happening to it.
 */
export function useStripCards(): StripCard[] {
  return [...STRIP_CARDS].sort(
    (a, b) => attention(b) - attention(a) || a.idleMinutes - b.idleMinutes,
  )
}

/** Becomes: invoke("running_count") — agents only; your own shell has no agent. */
export function useRunningCount(): number {
  return STRIP_CARDS.filter((c) => c.kind === 'agent').length
}
