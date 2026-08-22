import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunningPill } from '../../src/shell/RunningPill'

describe('shell/running-pill-counts', () => {
  it('renders both running and needs-you counts', () => {
    const { getByTestId } = render(() => <RunningPill running={8} needsYou={1} />)

    expect(getByTestId('running-pill').textContent).toContain('8 running')
    expect(getByTestId('running-pill').textContent).toContain('1 needs you')
    expect(getByTestId('running-pill-dot').className).toContain('pulse')
  })
})
