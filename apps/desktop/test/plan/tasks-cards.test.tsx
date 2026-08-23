import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanView } from '../../src/screens/plan/PlanView'

describe('planning tasks and cards', () => {
  it('renders editable rows and the card count', () => {
    const { getByTestId } = render(() => <PlanView />)
    fireEvent.click(getByTestId('plan-tab-tasks'))
    expect(getByTestId('tasks-cards-summary').textContent).toContain('cards')
  })
})
