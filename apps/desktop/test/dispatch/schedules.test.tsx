import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DispatchView } from '../../src/screens/dispatch/DispatchView'

describe('dispatch schedules', () => {
  it('renders schedule overlap and skipped outcome', () => {
    const { getByTestId } = render(() => <DispatchView tab="schedules" />)
    expect(getByTestId('schedule-outcome').textContent).toContain('Overlap is skipped')
  })
})
