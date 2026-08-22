import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunningPill } from '../../src/shell/RunningPill'

describe('shell/running-popover', () => {
  it('opens and closes the active-session popover', () => {
    const { getByRole, getByTestId, queryByRole } = render(() => (
      <RunningPill running={8} needsYou={1} />
    ))

    getByTestId('running-pill').click()
    expect(getByRole('dialog', { name: 'Active sessions' })).toBeTruthy()

    getByRole('button', { name: 'Close active sessions' }).click()
    expect(queryByRole('dialog', { name: 'Active sessions' })).toBeNull()
  })
})
