import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import PlanView from '../../src/screens/plan/PlanView'

describe('planning draft outputs', () => {
  it('renders recommendation and tools rail', () => {
    const { getByTestId } = render(() => <PlanView />)
    expect(getByTestId('plan-outputs').textContent).toContain('tool list')
  })
})
