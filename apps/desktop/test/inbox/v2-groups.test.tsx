import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2InboxView } from '../../src/screens/v2-dashboard'

describe('Inbox v2 groups', () => {
  it('labels action-required and completed groups', () => {
    const { getByTestId } = render(() => <V2InboxView />)
    expect(getByTestId('v2-inbox-tabs').querySelectorAll('[data-inbox-group]')).toHaveLength(2)
  })
})
