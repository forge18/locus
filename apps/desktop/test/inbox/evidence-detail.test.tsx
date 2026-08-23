import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2InboxView } from '../../src/screens/v2-dashboard'

describe('Inbox evidence detail', () => {
  it('labels evidence, why, and waiting cost', () => {
    const { getByTestId } = render(() => <V2InboxView />)
    expect(getByTestId('v2-inbox').querySelectorAll('[data-inbox-detail]')).toHaveLength(3)
  })
})
