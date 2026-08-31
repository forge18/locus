import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DispatchView } from '../../src/screens/dispatch/DispatchView'
import { configureProjectsStub } from '../projects/provider-stub'

configureProjectsStub();

describe('dispatch schedules', () => {
  it('renders schedule overlap and skipped outcome', () => {
    const { getByTestId } = render(() => <DispatchView tab="schedules" />)
    expect(getByTestId('schedule-outcome').textContent).toContain('Overlap is skipped')
  })
})
