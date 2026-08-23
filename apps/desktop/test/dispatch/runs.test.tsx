import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DispatchView } from '../../src/screens/dispatch/DispatchView'

describe('dispatch runs', () => {
  it('renders queue/run pause controls', () => {
    const { getByTestId } = render(() => <DispatchView tab="runs" />)
    expect(getByTestId('dispatch-pause-controls').textContent).toContain('Today')
  })
})
