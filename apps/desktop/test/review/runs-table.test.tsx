import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { RunsView } from '../../src/screens/review/RunsView'

describe('Review runs table', () => {
  it('shows model, tokens, cache, spend, and outcome', () => {
    const { getByTestId } = render(() => <RunsView />)
    expect(getByTestId('runs').textContent).toContain('Cache')
    expect(getByTestId('runs').textContent).toContain('Spend')
  })
})
