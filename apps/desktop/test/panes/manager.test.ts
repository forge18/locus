import { describe, expect, it } from 'vitest'
import { close, focus, minimize, panes, promote, resize, split, type Pane, type PaneTree } from '../../src/panes/manager'

const a: Pane = { id: 'a', kind: 'shell', focusedAt: 1 }
const b: Pane = { id: 'b', kind: 'agent', focusedAt: 2 }
const tree: PaneTree = { type: 'leaf', pane: a }

describe('panes/manager', () => {
  it('splits, resizes, focuses, and closes a pane tree', () => {
    const splitTree = split(tree, 'a', b, 'horizontal')
    expect(panes(splitTree)).toHaveLength(2)
    const resized = resize(splitTree, 'a', 3)
    expect(resized.type).toBe('split')
    if (resized.type === 'split') expect(resized.ratio).toBe(.9)
    expect(panes(focus(splitTree, 'a', 9))[0].focusedAt).toBe(9)
    expect(panes(close(splitTree, 'a')!)).toEqual([b])
  })
  it('minimizes without closing and promotes by demoting the least recent pane', () => {
    const layout = { focused: [a, b, { id: 'c', kind: 'editor' as const, focusedAt: 3 }, { id: 'd', kind: 'shell' as const, focusedAt: 4 }], strip: [{ id: 'e', kind: 'agent' as const, focusedAt: 5 }] }
    expect(minimize(layout, 'a').strip.map((pane) => pane.id)).toContain('a')
    const result = promote(layout, 'e', 10)
    expect(result.focused.map((pane) => pane.id)).toContain('e')
    expect(result.strip.map((pane) => pane.id)).toContain('a')
    expect([...result.focused, ...result.strip].map((pane) => pane.id).sort()).toEqual(['a', 'b', 'c', 'd', 'e'])
  })
})
