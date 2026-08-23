import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopInboxView } from '../../src/screens/desktop-dashboard'

describe('Inbox desktop groups', () => {
  it('labels action-required and completed groups', () => {
    const { getByTestId } = render(() => <DesktopInboxView />)
    expect(getByTestId('desktop-inbox-tabs').querySelectorAll('[data-inbox-group]')).toHaveLength(2)
  })
})
