import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanView } from '../../src/screens/plan/PlanView'

describe('planning conversation', () => {
  it('renders stage progress and an ACP live line', () => {
    const { getByTestId } = render(() => <PlanView />)
    expect(getByTestId('plan-stage-progress').textContent).toContain('Stage 5 of 9')
    expect(getByTestId('plan-live').textContent).not.toBe('')
  })
})
