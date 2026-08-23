import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { GuardrailsView } from '../../src/screens/settings/GuardrailsView'

describe('Guardrails', () => {
  it('renders stopping and parallelism controls', () => {
    const { getByTestId } = render(() => <GuardrailsView />)
    expect(getByTestId('settings-stepper-max-iterations').textContent).toContain('8')
    expect(getByTestId('settings-stepper-max-parallel-agents').textContent).toContain('6')
  })
})
