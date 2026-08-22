import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunningPill, type ActiveSession } from '../../src/shell/RunningPill'

const sessions: ActiveSession[] = [
  { id: 'active', label: 'Active', needsAttention: false, lastActivityAt: 30 },
  { id: 'waiting-old', label: 'Waiting old', needsAttention: true, lastActivityAt: 10 },
  { id: 'waiting-new', label: 'Waiting new', needsAttention: true, lastActivityAt: 20 },
]

describe('shell/running-popover-order', () => {
  it('sorts sessions by needs-attention and then descending activity', () => {
    const { getByTestId } = render(() => <RunningPill running={1} needsYou={2} sessions={sessions} />)

    getByTestId('running-pill').click()
    expect([...getByTestId('active-session-list').querySelectorAll('li')].map((item) => item.textContent)).toEqual([
      'Waiting new',
      'Waiting old',
      'Active',
    ])
  })
})
