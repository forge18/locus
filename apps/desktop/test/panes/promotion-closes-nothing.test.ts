import { expect, it } from 'vitest'
import { promote, type PaneLayout } from '../../src/panes/manager'

it('retains every pane when promotion demotes one to the strip', () => {
  const layout: PaneLayout = {
    focused: [1, 2, 3, 4].map((focusedAt) => ({ id: String(focusedAt), kind: 'agent' as const, focusedAt })),
    strip: [{ id: 'five', kind: 'agent', focusedAt: 5 }],
  }
  expect([...promote(layout, 'five', 10).focused, ...promote(layout, 'five', 10).strip]
    .map((pane) => pane.id).sort()).toEqual(['1', '2', '3', '4', 'five'])
})
