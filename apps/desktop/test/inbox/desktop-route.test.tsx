import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopInboxView } from '../../src/screens/desktop-dashboard'

describe('Inbox desktop route', () => {
  it('identifies the global inbox fixture route', () => {
    const { getByTestId } = render(() => <DesktopInboxView />)
    expect(getByTestId('desktop-inbox').getAttribute('data-desktop-route')).toBe('inbox')
  })
})
