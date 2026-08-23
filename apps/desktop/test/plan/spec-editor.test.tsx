import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanView } from '../../src/screens/plan/PlanView'

describe('spec editor', () => {
  it('renders outline, finding, and unsaved state', () => {
    const { getByTestId } = render(() => <PlanView />)
    fireEvent.click(getByTestId('plan-tab-spec'))
    expect(getByTestId('spec-outline').textContent).toContain('Conflict resolution')
    expect(getByTestId('spec-unsaved').textContent).toBe('unsaved')
  })
})
