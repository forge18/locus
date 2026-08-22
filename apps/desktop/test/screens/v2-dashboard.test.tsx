import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { V2DashboardView, V2InboxView } from '../../src/screens/v2-dashboard'
import { read, rules } from '../css'

const screenRules = rules(read('screens/screens.css'))
const rule = (selector: string) => screenRules.find((candidate) => candidate.selector === selector)

describe('screens/v2-dashboard', () => {
  it('renders the global Inbox fixture with its response queue and selected gate', () => {
    const { container, getByTestId, getByText } = render(() => <V2InboxView />)

    expect(getByTestId('v2-inbox')).toBeTruthy()
    expect(getByTestId('v2-inbox-tabs').textContent).toContain('To do')
    expect(getByTestId('v2-inbox-tabs').textContent).toContain('Completed')
    expect(getByTestId('v2-inbox-budget').textContent).toContain('3 / 6 per hour')
    expect(getByTestId('v2-inbox-items').querySelectorAll('[data-inbox-item]').length).toBe(3)
    expect(getByText('Keyframe extraction for recordings')).toBeTruthy()
    expect(getByText('Approve & release the loop')).toBeTruthy()
    expect(container.textContent).toContain('No tokens burn while blocked.')
  })

  it('renders the global Dashboard fixture and keeps magnitude charts on the data ramp', () => {
    const { getByTestId, getByText } = render(() => <V2DashboardView />)

    expect(getByTestId('v2-dashboard')).toBeTruthy()
    expect(getByText('All projects')).toBeTruthy()
    expect(getByTestId('v2-dashboard-range').textContent).toContain('14d')
    expect(getByTestId('v2-dashboard-running').textContent).toContain('8')
    expect(getByTestId('v2-token-chart').querySelectorAll('[data-token-day]').length).toBe(14)
    expect(getByTestId('v2-model-scorecard').querySelectorAll('tbody tr').length).toBe(4)
    expect(getByTestId('v2-dashboard-counters').querySelectorAll('[data-dashboard-counter]').length).toBe(4)
    expect(rule('.v2-magnitude-fill')?.body).toContain('var(--data-')
    expect(rule('.v2-magnitude-fill')?.body).not.toContain('var(--ac)')
    expect(rule('.v2-magnitude-fill')?.body).not.toContain('var(--ac2)')
  })
})
