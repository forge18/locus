import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Breadcrumb, stepState } from '../../src/screens/plan/Breadcrumb'
import { PlanView } from '../../src/screens/plan/PlanView'
import { PLAN_STEPS, usePlans } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!

describe('plan/breadcrumb', () => {
  it('shows all eight steps, in order', () => {
    const { getByTestId } = render(() => <Breadcrumb current="Audit" />)
    expect([...getByTestId('breadcrumb').children].map((c) => c.textContent?.replace(/^\d/, ''))).toEqual(
      [...PLAN_STEPS],
    )
  })

  it('has three distinct states', () => {
    const { getByTestId } = render(() => <Breadcrumb current="Audit" />)
    const states = [...getByTestId('breadcrumb').children].map((c) => c.getAttribute('data-state'))
    expect(new Set(states)).toEqual(new Set(['done', 'current', 'ahead']))
  })

  it('marks everything before the current step done, and after it ahead', () => {
    for (const step of PLAN_STEPS) {
      expect(stepState(step, 'Audit'), step).toBe(
        PLAN_STEPS.indexOf(step) < 4 ? 'done' : PLAN_STEPS.indexOf(step) === 4 ? 'current' : 'ahead',
      )
    }
  })

  it('checks the done steps in --ok, pills the current one in accent, dims the rest', () => {
    expect(rule('.crumb-done').body).toContain('color: var(--status-success)')
    expect(rule('.crumb-current').body).toContain('color: var(--action-attention)')
    expect(rule('.crumb-current').body).toContain('box-shadow: var(--ring-sel-soft)')
    expect(rule('.crumb').body).toContain('color: var(--text-muted)')

    const { getByTestId } = render(() => <Breadcrumb current="Audit" />)
    expect(getByTestId('crumb-inputs').querySelector('use')!.getAttribute('href')).toBe('#ph-check')
    expect(getByTestId('crumb-audit').getAttribute('aria-current')).toBe('step')
  })

  it('derives the current step from the plan, not from the markup', () => {
    const { getByTestId } = render(() => <PlanView />)
    const selected = usePlans()[0]
    expect(selected.step).toBe('Audit')
    expect(getByTestId(`crumb-${selected.step.toLowerCase()}`).getAttribute('data-state')).toBe(
      'current',
    )

    // A different plan moves it, with no markup change.
    getByTestId(`plan-card-${usePlans()[1].id}`).click()
    expect(getByTestId('crumb-synthesise').getAttribute('data-state')).toBe('current')
    expect(getByTestId('crumb-audit').getAttribute('data-state')).toBe('ahead')
  })
})
