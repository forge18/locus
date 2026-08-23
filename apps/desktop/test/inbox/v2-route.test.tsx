import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2InboxView } from '../../src/screens/v2-dashboard'

describe('Inbox v2 route', () => {
  it('identifies the global inbox fixture route', () => {
    const { getByTestId } = render(() => <V2InboxView />)
    expect(getByTestId('v2-inbox').getAttribute('data-v2-route')).toBe('inbox')
  })
})
