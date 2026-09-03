import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it, vi } from 'vitest'
import { GraphRenderer } from '../../src/workflow-canvas/GraphRenderer'

const nodes = [
  { id: 'a', label: 'Alpha', x: 10, y: 10 },
  { id: 'b', label: 'Beta', x: 40, y: 10 },
]

describe('canvas/graph-keyboard', () => {
  it('is a group, not an image — clickable nodes stay in the a11y tree', () => {
    const { getByTestId } = render(() => (
      <GraphRenderer nodes={nodes} edges={[]} width={100} height={50} onSelect={() => {}} />
    ))
    const svg = getByTestId('graph-svg')
    expect(svg.getAttribute('role')).toBe('group')
    expect(svg.getAttribute('aria-label')).toBe('Graph')
  })

  it('names, focuses, and activates each node with Enter or Space', () => {
    const onSelect = vi.fn()
    const { getByTestId } = render(() => (
      <GraphRenderer nodes={nodes} edges={[]} width={100} height={50} onSelect={onSelect} />
    ))
    const node = getByTestId('graph-node-a')
    expect(node.getAttribute('role')).toBe('button')
    expect(node.getAttribute('tabindex')).toBe('0')
    expect(node.getAttribute('aria-label')).toBe('Alpha')
    fireEvent.keyDown(node, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledWith('a')
    fireEvent.keyDown(node, { key: ' ' })
    expect(onSelect).toHaveBeenCalledWith('a')
    fireEvent.keyDown(node, { key: 'ArrowRight' })
    expect(onSelect).toHaveBeenCalledTimes(2)
  })

  it('is a static picture when nothing is selectable', () => {
    const { getByTestId } = render(() => (
      <GraphRenderer nodes={nodes} edges={[]} width={100} height={50} />
    ))
    const node = getByTestId('graph-node-a')
    expect(node.getAttribute('role')).toBe(null)
    expect(node.getAttribute('tabindex')).toBe(null)
    expect(node.getAttribute('aria-label')).toBe('Alpha')
  })
})
