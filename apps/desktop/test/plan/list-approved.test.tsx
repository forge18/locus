import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { usePlans } from '../../src/data/plan'
import { read, rules } from '../css'

const mount = () => render(() => <PlanView />)
const approved = usePlans().filter((p) => p.state === 'approved')

describe('plan/list-approved', () => {
  it('dims the approved cards', () => {
    const { getByTestId } = mount()
    for (const plan of approved) {
      expect(getByTestId(`plan-card-${plan.id}`).className, plan.id).toContain('plan-card-approved')
    }
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.plan-card-approved')!.body,
    ).toMatch(/opacity:\s*\.62/)
  })

  it('states what landed, in --ok', () => {
    const { getByTestId } = mount()
    const step = getByTestId(`plan-card-step-${approved[0].id}`)
    expect(step.textContent).toBe('8 tasks landed')
    expect(step.className).toContain('plan-card-landed')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.plan-card-landed')!.body,
    ).toContain('color: var(--status-success)')
  })

  it('shows no spinner on an approved card — it is finished, not running', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`plan-card-${approved[0].id}`).querySelector('use')).toBe(null)
  })

  it('leaves them reachable, dimmed rather than hidden', () => {
    const { getByTestId } = mount()
    const card = getByTestId(`plan-card-${approved[0].id}`)
    card.click()
    expect(card.getAttribute('aria-selected')).toBe('true')
  })
})
