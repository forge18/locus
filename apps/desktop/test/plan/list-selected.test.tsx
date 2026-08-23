import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { useDefaultPlanId, usePlans } from '../../src/data/plan'
import { read, rules } from '../css'

const mount = () => render(() => <PlanView />)
const selected = usePlans().find((p) => p.id === useDefaultPlanId())!

describe('plan/list-selected', () => {
  it('marks exactly one card selected', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('plan-list').querySelectorAll('[aria-selected="true"]')
    expect(marked.length).toBe(1)
    expect(marked[0].getAttribute('data-testid')).toBe(`plan-card-${selected.id}`)
  })

  it('paints it --sf2 with the accent inset ring', () => {
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === ".plan-card[aria-selected='true']",
    )!
    expect(rule.body).toContain('background: var(--surface-selected)')
    expect(rule.body).toContain('box-shadow: var(--ring-sel)')
  })

  it('carries the circle-notch on an in-progress card', () => {
    const { getByTestId } = mount()
    expect(
      getByTestId(`plan-card-${selected.id}`).querySelector('use')!.getAttribute('href'),
    ).toBe('#ph-circle-notch')
  })

  it('shows the step line', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`plan-card-step-${selected.id}`).textContent).toBe('step 5 · audit')
  })

  it('right-aligns the project', () => {
    const { getByTestId } = mount()
    const project = getByTestId(`plan-card-project-${selected.id}`)
    expect(project.textContent).toBe('loom-db')
    expect(project.className).toContain('plan-card-project')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.plan-card-project')!.body,
    ).toContain('margin-left: auto')
  })

  it('moves the selection and the conversation together', () => {
    const { getByTestId } = mount()
    const other = usePlans().find((p) => p.id !== selected.id)!
    getByTestId(`plan-card-${other.id}`).click()
    expect(getByTestId('plan-title').textContent).toBe(other.title)
  })
})
