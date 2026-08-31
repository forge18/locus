import { expect, it } from 'vitest'
import { promote, type PaneLayout } from '../../src/panes/manager'

it('caps focused panes at four and leaves overflow in the strip', () => {
  const layout: PaneLayout = {
    focused: [1, 2, 3, 4].map((focusedAt) => ({ id: String(focusedAt), kind: 'agent' as const, focusedAt })),
    strip: [{ id: 'five', kind: 'agent', focusedAt: 5 }],
  }
  const next = promote(layout, 'five', 10)
  expect(next.focused).toHaveLength(4)
  expect(next.strip).toHaveLength(1)
})
