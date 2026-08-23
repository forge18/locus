import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopInboxView } from '../../src/screens/desktop-dashboard'

describe('Inbox evidence detail', () => {
  it('labels evidence, why, and waiting cost', () => {
    const { getByTestId } = render(() => <DesktopInboxView />)
    expect(getByTestId('desktop-inbox').querySelectorAll('[data-inbox-detail]')).toHaveLength(3)
  })
})
