import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { GuardrailsView } from '../../src/screens/settings/GuardrailsView'

describe('parallelism settings', () => {
  it('renders global and per-project cap controls', () => {
    const { getByTestId } = render(() => <GuardrailsView />)
    expect(getByTestId('parallelism-controls').textContent).toContain('Max parallel agents')
  })
})
