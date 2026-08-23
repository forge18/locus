import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DesktopDashboardView, DesktopInboxView } from '../../src/screens/desktop-dashboard'
import { read, rules } from '../css'

const screenRules = rules(read('screens/screens.css'))
const rule = (selector: string) => screenRules.find((candidate) => candidate.selector === selector)

describe('screens/desktop-dashboard', () => {
  it('renders the global Inbox fixture with its response queue and selected gate', () => {
    const { container, getByTestId, getByText } = render(() => <DesktopInboxView />)

    expect(getByTestId('desktop-inbox')).toBeTruthy()
    expect(getByTestId('desktop-inbox-tabs').textContent).toContain('To do')
    expect(getByTestId('desktop-inbox-tabs').textContent).toContain('Completed')
    expect(getByTestId('desktop-inbox-budget').textContent).toContain('3 / 6 per hour')
    expect(getByTestId('desktop-inbox-items').querySelectorAll('[data-inbox-item]').length).toBe(3)
    expect(getByText('Keyframe extraction for recordings')).toBeTruthy()
    expect(getByText('Approve & release the loop')).toBeTruthy()
    expect(container.textContent).toContain('No tokens burn while blocked.')
  })

  it('renders the global Dashboard fixture and keeps magnitude charts on the data ramp', () => {
    const { getByTestId, getByText } = render(() => <DesktopDashboardView />)

    expect(getByTestId('desktop-dashboard')).toBeTruthy()
    expect(getByText('All projects')).toBeTruthy()
    expect(getByTestId('desktop-dashboard-range').textContent).toContain('14d')
    expect(getByTestId('desktop-dashboard-running').textContent).toContain('8')
    expect(getByTestId('desktop-token-chart').querySelectorAll('[data-token-day]').length).toBe(14)
    expect(getByTestId('desktop-model-scorecard').querySelectorAll('tbody tr').length).toBe(4)
    expect(getByTestId('desktop-dashboard-counters').querySelectorAll('[data-dashboard-counter]').length).toBe(4)
    expect(rule('.desktop-magnitude-fill')?.body).toContain('var(--data-')
    expect(rule('.desktop-magnitude-fill')?.body).not.toContain('var(--action-attention)')
    expect(rule('.desktop-magnitude-fill')?.body).not.toContain('var(--ac2)')
  })
})
