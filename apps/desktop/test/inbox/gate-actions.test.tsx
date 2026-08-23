import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopInboxView } from '../../src/screens/desktop-dashboard'

describe('Inbox gate actions', () => {
  it('labels approve and send-back actions', () => {
    const { getByTestId } = render(() => <DesktopInboxView />)
    expect(getByTestId('desktop-inbox').querySelectorAll('[data-inbox-gate-action]')).toHaveLength(2)
  })
})
