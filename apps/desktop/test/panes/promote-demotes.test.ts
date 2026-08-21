import { expect, it } from 'vitest'
import { promote, type PaneLayout } from '../../src/panes/manager'

it('promotes a strip pane by demoting the least recently focused pane', () => {
  const layout: PaneLayout = {
    focused: [1, 2, 3, 4].map((focusedAt) => ({ id: String(focusedAt), kind: 'shell' as const, focusedAt })),
    strip: [{ id: 'five', kind: 'agent', focusedAt: 5 }],
  }

  const next = promote(layout, 'five', 10)
  expect(next.focused.map((pane) => pane.id)).toEqual(['2', '3', '4', 'five'])
  expect(next.strip.map((pane) => pane.id)).toEqual(['1'])
})
