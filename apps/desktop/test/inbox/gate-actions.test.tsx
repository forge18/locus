import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2InboxView } from '../../src/screens/v2-dashboard'

describe('Inbox gate actions', () => {
  it('labels approve and send-back actions', () => {
    const { getByTestId } = render(() => <V2InboxView />)
    expect(getByTestId('v2-inbox').querySelectorAll('[data-inbox-gate-action]')).toHaveLength(2)
  })
})
