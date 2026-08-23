import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DispatchView } from '../../src/screens/dispatch/DispatchView'

describe('dispatch autorun', () => {
  it('renders autorun rationale and review debt', () => {
    const { getByTestId } = render(() => <DispatchView tab="autorun" />)
    expect(getByTestId('autorun-review-debt').textContent).toContain('review slots')
  })
})
