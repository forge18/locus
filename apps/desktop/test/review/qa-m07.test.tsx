import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { QAView } from '../../src/screens/review/QAView'

describe('M0.7 QA', () => {
  it('renders four pluggable groups, schedule choices, and footer', () => {
    const { getByTestId, getByText } = render(() => <QAView projectId="tapestry" />)
    expect(getByTestId('qa-group-unit-tests')).toBeTruthy()
    expect(getByTestId('qa-group-linters')).toBeTruthy()
    expect(getByTestId('qa-group-lsp')).toBeTruthy()
    expect(getByTestId('qa-group-agent-reviews')).toBeTruthy()
    expect(getByText('Manual')).toBeTruthy()
    expect(getByText('Push')).toBeTruthy()
    expect(getByText('Hourly')).toBeTruthy()
    expect(getByText('Daily')).toBeTruthy()
    expect(getByText(/Not real-time/)).toBeTruthy()
  })

  it('keeps a finding visible after sending it to Inbox', () => {
    const { getByTestId } = render(() => <QAView />)
    const finding = getByTestId('qa-finding-qa-test-1')
    fireEvent.click(finding.querySelector('button')!)
    expect(finding).toBeTruthy()
    expect(finding.textContent).toContain('Sent to Inbox')
  })
})
