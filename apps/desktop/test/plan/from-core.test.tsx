import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { PlanView } from '../../src/screens/plan/PlanView'

describe('plan/from-core', () => {
  it('renders the ACP planning conversation and durable draft outputs', () => {
    const { getByTestId } = render(() => <PlanView />)
    expect(getByTestId('plan-conversation')).toBeTruthy()
    expect(getByTestId('plan-messages')).toBeTruthy()
    expect(getByTestId('plan-outputs')).toBeTruthy()
    expect(getByTestId('plan-acp').textContent).toContain('ACP')
  })
})
