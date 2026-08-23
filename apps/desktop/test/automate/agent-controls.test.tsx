import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { AgentsView } from '../../src/screens/automate/AgentsView'

describe('agent controls', () => {
  it('renders handoff controls for attention states', () => {
    const { getByTestId } = render(() => <AgentsView />)
    expect(getByTestId('guardrail-handoff').textContent).toContain('Hand off')
  })
})
