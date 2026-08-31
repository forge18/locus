import { expect, it } from 'vitest'
import { minimize, type PaneLayout } from '../../src/panes/manager'

it('moves a focused pane into the strip', () => {
  const layout: PaneLayout = { focused: [{ id: 'one', kind: 'agent', focusedAt: 1 }], strip: [] }
  expect(minimize(layout, 'one')).toEqual({ focused: [], strip: [{ id: 'one', kind: 'agent', focusedAt: 1 }] })
})
