import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanView } from '../../src/screens/plan/PlanView'

const mount = () => render(() => <PlanView />)

describe('screens/desktop-plan', () => {
  it('renders the nine-stage planning workspace with conversation selected', () => {
    const { getByTestId } = mount()

    expect(getByTestId('plan-workspace-tabs').textContent).toContain('Conversation')
    expect(getByTestId('plan-workspace-tabs').textContent).toContain('Spec')
    expect(getByTestId('plan-workspace-tabs').textContent).toContain('Tasks & cards')
    expect(getByTestId('breadcrumb').children).toHaveLength(9)
    expect(getByTestId('crumb-decompose').textContent).toContain('Decompose')
    expect(getByTestId('plan-conversation')).toBeTruthy()
  })

  it('renders the editable spec with stable requirement ids and its inline finding', () => {
    const { getByTestId } = mount()
    fireEvent.click(getByTestId('plan-tab-spec'))

    expect(getByTestId('plan-spec')).toBeTruthy()
    expect(getByTestId('requirement-R-07').querySelector('textarea')?.value).toContain('verified_at')
    expect(getByTestId('requirement-finding-R-07').textContent).toContain('missed question')
    expect(getByTestId('spec-unsaved')).toBeTruthy()

    fireEvent.click(getByTestId('resolve-finding-R-07'))
    expect(getByTestId('requirement-finding-R-07').textContent).toContain('Finding resolved')
  })

  it('maps editable tasks to cards and recalculates the card count', () => {
    const { getByTestId } = mount()
    fireEvent.click(getByTestId('plan-tab-tasks'))

    expect(getByTestId('plan-tasks')).toBeTruthy()
    expect(getByTestId('granularity-spec-carve-outs').getAttribute('aria-checked')).toBe('true')
    expect(getByTestId('task-card-count').textContent).toContain('3 cards')

    fireEvent.click(getByTestId('task-card-toggle-T-01'))
    expect(getByTestId('task-card-count').textContent).toContain('4 cards')

    fireEvent.click(getByTestId('granularity-every-task'))
    expect(getByTestId('task-card-count').textContent).toContain('4 cards')
  })
})
