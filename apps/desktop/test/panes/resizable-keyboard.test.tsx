import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { Resizable } from '../../src/panes/Resizable'

const mount = (side: 'left' | 'right' = 'right') =>
  render(() => (
    <Resizable width={300} min={200} max={400} side={side}>
      <p>pane</p>
    </Resizable>
  ))

describe('panes/resizable-keyboard', () => {
  it('focuses the separator and reports its resize range', () => {
    const { getByTestId } = mount()
    const handle = getByTestId('resizable-handle')
    expect(handle.getAttribute('role')).toBe('separator')
    expect(handle.getAttribute('tabindex')).toBe('0')
    expect(handle.getAttribute('aria-valuemin')).toBe('200')
    expect(handle.getAttribute('aria-valuemax')).toBe('400')
    expect(handle.getAttribute('aria-valuenow')).toBe('300')
  })

  it('widens a right-edge pane with ArrowRight and narrows with ArrowLeft', () => {
    const { getByTestId } = mount('right')
    const handle = getByTestId('resizable-handle')
    fireEvent.keyDown(handle, { key: 'ArrowRight' })
    expect(handle.getAttribute('aria-valuenow')).toBe('316')
    fireEvent.keyDown(handle, { key: 'ArrowLeft' })
    expect(handle.getAttribute('aria-valuenow')).toBe('300')
  })

  it('mirrors the arrows for a left-edge handle', () => {
    const { getByTestId } = mount('left')
    const handle = getByTestId('resizable-handle')
    fireEvent.keyDown(handle, { key: 'ArrowLeft' })
    expect(handle.getAttribute('aria-valuenow')).toBe('316')
  })

  it('stays within the clamp and ignores other keys', () => {
    const { getByTestId } = mount()
    const handle = getByTestId('resizable-handle')
    for (let i = 0; i < 20; i += 1) fireEvent.keyDown(handle, { key: 'ArrowRight' })
    expect(handle.getAttribute('aria-valuenow')).toBe('400')
    fireEvent.keyDown(handle, { key: 'ArrowUp' })
    expect(handle.getAttribute('aria-valuenow')).toBe('400')
  })
})
