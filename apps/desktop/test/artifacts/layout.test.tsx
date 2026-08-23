import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ArtifactsView } from '../../src/screens/review/ArtifactsView'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <ArtifactsView />)

describe('artifacts/layout', () => {
  it('is three panes', () => {
    const { getByTestId } = mount()
    expect(getByTestId('artifact-list')).toBeTruthy()
    expect(getByTestId('artifact-view')).toBeTruthy()
    expect(getByTestId('comment-rail')).toBeTruthy()
  })

  it('starts the list at 222px and the rail at 306px', () => {
    const { getByTestId } = mount()
    expect((getByTestId('artifact-list') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('222px')
    expect((getByTestId('comment-rail') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('306px')
  })

  it('lets the viewer take the rest', () => {
    expect(rule('.artifact-view').body).toContain('flex: 1')
    expect(rule('.artifact-view').body).toContain('min-width: 0')
  })

  it('hairlines both seams', () => {
    expect(rule('.artifact-list').body).toContain('border-right: 1px solid var(--border-subtle)')
    expect(rule('.comment-rail').body).toContain('border-left: 1px solid var(--border-subtle)')
  })

  it('scrolls the diff and the thread on their own', () => {
    expect(rule('.udiff').body).toContain('overflow: auto')
    expect(rule('.comment-rail-body').body).toContain('overflow: auto')
  })
})
