import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { createNavStore } from '../../src/nav'
import { InboxView } from '../../src/screens/inbox/InboxView'

describe('inbox/reflection-queue', () => {
  it('shares the human review queue with calibration proposals', async () => {
    const { getByTestId, getByText } = render(() => <InboxView nav={createNavStore({ view: 'inbox' })} />)
    const card = getByTestId('inbox-card-in-reflection-1')
    expect(card.getAttribute('data-kind')).toBe('reflection')
    await fireEvent.click(card)
    expect(getByText('Reflection proposal')).toBeTruthy()
    expect(getByTestId('inbox-detail').textContent).toContain('nothing applies automatically')
  })
})
