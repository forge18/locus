import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DispatchView } from '../../src/screens/dispatch/DispatchView'

describe('stop all', () => {
  it('renders confirmation and stopped restore action', () => {
    const { getByText, getByTestId } = render(() => <DispatchView tab="autorun" />)
    fireEvent.click(getByText('Stop all'))
    fireEvent.click(getByText(/Stop all —/))
    expect(getByTestId('stop-all-restore').textContent).toContain('Restore')
  })
})
