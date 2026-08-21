import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { read } from '../css'

const mount = () => {
  document.body.innerHTML = ''
  const root = document.createElement('div')
  root.id = 'root'
  document.body.appendChild(root)
  return render(() => <PlanView />, { container: root })
}

describe('plan/scope-is-inline', () => {
  it('renders inside the message flow', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-messages').contains(getByTestId('scope-decision'))).toBe(true)
  })

  it('sits between two messages, in sequence', () => {
    const { getByTestId } = mount()
    const children = [...getByTestId('plan-messages').children]
    const index = children.indexOf(getByTestId('scope-decision'))
    expect(index).toBeGreaterThan(0)
    expect(index).toBeLessThan(children.length - 1)
    expect(children[index - 1].className).toContain('msg')
  })

  it('is not a modal — nothing is portalled and nothing overlays', () => {
    mount()
    expect(document.querySelector('.overlay')).toBe(null)
    expect(document.querySelector('.sheet')).toBe(null)
    expect(document.querySelector('[role="dialog"]')).toBe(null)
  })

  it('is not a gate — the screen never reaches for the dialog components', () => {
    const source = [
      read('screens/plan/ScopeDecision.tsx'),
      read('screens/plan/PlanView.tsx'),
    ].join('\n')
    expect(source).not.toMatch(/Sheet|Dialog|Portal|openDetail/)
  })

  it('does not block the conversation — the live line still renders below it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-live')).toBeTruthy()
  })
})
