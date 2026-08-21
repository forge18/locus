import { describe, expect, it } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <DevelopView />)

describe('develop/layout', () => {
  it('is three columns', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree')).toBeTruthy()
    expect(getByTestId('dev-editor')).toBeTruthy()
    expect(getByTestId('git-panel')).toBeTruthy()
  })

  it('starts the tree at 206px and the git panel at 252px', () => {
    const { getByTestId } = mount()
    expect((getByTestId('dev-tree') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('206px')
    expect((getByTestId('git-panel') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('252px')
  })

  it('lets the editor take the rest', () => {
    expect(rule('.dev-editor').body).toContain('flex: 1')
    expect(rule('.dev-editor').body).toContain('min-width: 0')
  })

  it('gives both side columns a drag handle', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tree-handle').getAttribute('role')).toBe('separator')
    expect(getByTestId('git-panel-handle').getAttribute('role')).toBe('separator')
  })

  it('resizes on drag, so the drawn widths are preferences and not constants', () => {
    const { getByTestId } = mount()
    const tree = getByTestId('dev-tree') as HTMLElement
    expect(tree.getAttribute('data-dragged')).toBe(null)

    fireEvent.pointerDown(getByTestId('dev-tree-handle'), { clientX: 0 })
    fireEvent.pointerMove(document, { clientX: 60 })
    expect(tree.style.getPropertyValue('--pane-w')).toBe('266px')
    expect(tree.getAttribute('data-dragged')).toBe('true')
    fireEvent.pointerUp(document)
  })

  it('drags the git panel the other way, because its handle is on the left', () => {
    const { getByTestId } = mount()
    const panel = getByTestId('git-panel') as HTMLElement
    fireEvent.pointerDown(getByTestId('git-panel-handle'), { clientX: 0 })
    fireEvent.pointerMove(document, { clientX: -40 })
    expect(panel.style.getPropertyValue('--pane-w')).toBe('292px')
    fireEvent.pointerUp(document)
  })

  it('clamps a drag rather than letting a column vanish', () => {
    const { getByTestId } = mount()
    const tree = getByTestId('dev-tree') as HTMLElement
    fireEvent.pointerDown(getByTestId('dev-tree-handle'), { clientX: 0 })
    fireEvent.pointerMove(document, { clientX: -400 })
    expect(tree.style.getPropertyValue('--pane-w')).toBe('160px')
    fireEvent.pointerUp(document)
  })
})
