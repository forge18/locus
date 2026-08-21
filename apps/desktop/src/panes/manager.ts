export type PaneKind = 'shell' | 'agent' | 'editor'

export interface Pane {
  id: string
  kind: PaneKind
  runId?: string
  focusedAt: number
}

export type PaneTree =
  | { type: 'leaf'; pane: Pane }
  | { type: 'split'; direction: 'horizontal' | 'vertical'; ratio: number; first: PaneTree; second: PaneTree }

const leaf = (pane: Pane): PaneTree => ({ type: 'leaf', pane })

export function split(tree: PaneTree, target: string, pane: Pane, direction: 'horizontal' | 'vertical'): PaneTree {
  if (tree.type === 'leaf') {
    return tree.pane.id === target
      ? { type: 'split', direction, ratio: 0.5, first: tree, second: leaf(pane) }
      : tree
  }
  return { ...tree, first: split(tree.first, target, pane, direction), second: split(tree.second, target, pane, direction) }
}

export function resize(tree: PaneTree, target: string, ratio: number): PaneTree {
  if (tree.type === 'leaf') return tree
  const inside = panes(tree).some((pane) => pane.id === target)
  return inside ? { ...tree, ratio: Math.min(0.9, Math.max(0.1, ratio)) } : { ...tree, first: resize(tree.first, target, ratio), second: resize(tree.second, target, ratio) }
}

export function close(tree: PaneTree, target: string): PaneTree | undefined {
  if (tree.type === 'leaf') return tree.pane.id === target ? undefined : tree
  const first = close(tree.first, target)
  const second = close(tree.second, target)
  if (!first) return second
  if (!second) return first
  return { ...tree, first, second }
}

export function focus(tree: PaneTree, target: string, at = Date.now()): PaneTree {
  if (tree.type === 'leaf') return tree.pane.id === target ? { type: 'leaf', pane: { ...tree.pane, focusedAt: at } } : tree
  return { ...tree, first: focus(tree.first, target, at), second: focus(tree.second, target, at) }
}

export function panes(tree: PaneTree): Pane[] {
  return tree.type === 'leaf' ? [tree.pane] : [...panes(tree.first), ...panes(tree.second)]
}

export interface PaneLayout { focused: Pane[]; strip: Pane[] }

export function minimize(layout: PaneLayout, id: string): PaneLayout {
  const pane = layout.focused.find((item) => item.id === id)
  return pane ? { focused: layout.focused.filter((item) => item.id !== id), strip: [...layout.strip, pane] } : layout
}

export function promote(layout: PaneLayout, id: string, at = Date.now()): PaneLayout {
  const pane = layout.strip.find((item) => item.id === id)
  if (!pane) return layout
  const promoted = { ...pane, focusedAt: at }
  const remaining = layout.strip.filter((item) => item.id !== id)
  if (layout.focused.length < 4) return { focused: [...layout.focused, promoted], strip: remaining }
  const leastRecent = layout.focused.reduce((least, item) => item.focusedAt < least.focusedAt ? item : least)
  return { focused: [...layout.focused.filter((item) => item.id !== leastRecent.id), promoted], strip: [...remaining, leastRecent] }
}
