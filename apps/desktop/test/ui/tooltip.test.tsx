import { describe, expect, it } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { Tooltip } from '../../src/ui/Tooltip'
import { read, rules } from '../css'

describe('ui/tooltip', () => {
  it('stays closed until the trigger is reached', () => {
    render(() => (
      <Tooltip content="waiting: gate — not idle">
        <span>hourglass</span>
      </Tooltip>
    ))
    expect(document.querySelector('[data-testid="tooltip-content"]')).toBe(null)
  })

  it('opens when the trigger is reached', async () => {
    const { getByTestId } = render(() => (
      <Tooltip content="waiting: gate — not idle" openDelay={0}>
        <span>hourglass</span>
      </Tooltip>
    ))
    const trigger = getByTestId('tooltip-trigger')
    trigger.dispatchEvent(new MouseEvent('pointerenter', { bubbles: false }))
    await waitFor(() =>
      expect(document.querySelector('[data-testid="tooltip-content"]')?.textContent).toBe(
        'waiting: gate — not idle',
      ),
    )
  })

  it('is styled from tokens, on the deep ground', () => {
    const rule = rules(read('ui/ui.css')).find((r) => r.selector === '.tooltip')!
    expect(rule.body).toContain('background: var(--surface-chrome)')
    expect(rule.body).toContain('color: var(--text-primary)')
    expect(rule.body).toContain('border: 1px solid var(--border-strong)')
  })

  it('renders its trigger content rather than replacing it', () => {
    const { getByText } = render(() => (
      <Tooltip content="tip">
        <span>hourglass</span>
      </Tooltip>
    ))
    expect(getByText('hourglass')).toBeTruthy()
  })
})
