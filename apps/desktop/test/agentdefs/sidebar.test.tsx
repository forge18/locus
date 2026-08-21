import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { read, rules } from '../css'

const mount = (onNavigate: (v: string) => void = () => {}) =>
  render(() => <AgentDefsView onNavigate={onNavigate as never} />)

describe('agentdefs/sidebar', () => {
  it('starts at 196px', () => {
    const { getByTestId } = mount()
    expect((getByTestId('agentdefs-side') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('196px')
  })

  it('hairlines the seam', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.agentdefs-side')!.body,
    ).toContain('border-right: 1px solid var(--line)')
  })

  it('leads with the ← Extensions back link, in accent', () => {
    const { getByTestId } = mount()
    const back = getByTestId('agentdefs-back')
    expect(back.textContent).toContain('Extensions')
    expect(back.querySelector('use')!.getAttribute('href')).toBe('#ph-arrow-left')
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.agentdefs-back',
    )!.body
    expect(body).toContain('color: var(--ac)')
    expect(body).toContain('font-size: var(--t-meta)')
  })

  it('goes back to Extensions when followed', () => {
    const landed: string[] = []
    const { getByTestId } = mount((v) => landed.push(v))
    getByTestId('agentdefs-back').click()
    expect(landed).toEqual(['extensions'])
  })

  it('is headed AGENT DEFINITIONS under the back link', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-list-title').textContent).toBe('Agent definitions')
    expect(
      getByTestId('agentdefs-back').compareDocumentPosition(getByTestId('agentdefs-list-title')) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
