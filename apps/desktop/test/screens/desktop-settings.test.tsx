import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GuardrailsView } from '../../src/screens/settings/GuardrailsView'
import { read, rules } from '../css'

const rule = (selector: string) =>
  rules(read('screens/screens.css')).find((entry) => entry.selector === selector)!

const mount = () => render(() => <GuardrailsView />)

describe('screens/desktop-settings', () => {
  it('renders the install-scoped Settings rail with Guardrails selected', () => {
    const { getByTestId } = mount()

    expect(getByTestId('settings')).toBeTruthy()
    expect(getByTestId('settings-rail')).toBeTruthy()
    expect(getByTestId('settings-nav-guardrails').getAttribute('aria-current')).toBe('page')
    expect(getByTestId('settings-install-note').textContent).toContain('per install')
  })

  it('shows stopping, parallelism, change-size, and permission defaults', () => {
    const { getByTestId } = mount()

    for (const section of ['stopping', 'change-size', 'permissions']) {
      expect(getByTestId(`settings-section-${section}`), section).toBeTruthy()
    }
    expect(getByTestId('parallelism-controls')).toBeTruthy()
    expect(getByTestId('settings-value-max-iterations').textContent).toBe('8')
    expect(getByTestId('settings-value-token-budget').textContent).toBe('120k')
    expect(getByTestId('settings-value-max-parallel-agents').textContent).toBe('6')
    expect(getByTestId('settings-value-max-per-project').textContent).toBe('3')
  })

  it('explains all priority choices and preserves the preemption handoff boundary', () => {
    const { getByTestId } = mount()

    expect(getByTestId('settings-priority-method').textContent).toContain('plan order')
    expect(getByTestId('settings-priority-method').textContent).toContain('unblocks-most')
    expect(getByTestId('settings-priority-method').textContent).toContain('shortest first')
    expect(getByTestId('settings-value-tie-break').textContent).toContain('longest waiting')
    expect(getByTestId('settings-preempt-note').textContent).toContain('handoff, not its context')
    expect(getByTestId('settings-toggle-preempt').getAttribute('data-on')).toBe('false')
  })

  it('visibly marks the active theme option', () => {
    window.localStorage.clear()
    const { getByRole } = mount()
    const dark = getByRole('button', { name: 'Dark' })
    const light = getByRole('button', { name: 'Light' })

    expect(dark.classList.contains('settings-theme-selected')).toBe(true)
    expect(dark.getAttribute('aria-pressed')).toBe('true')
    expect(light.classList.contains('settings-theme-selected')).toBe(false)
    expect(light.getAttribute('aria-pressed')).toBe('false')

    const selected = rule('.settings-theme-selected').body
    expect(selected).toContain('background: var(--surface-selected)')
    expect(selected).toContain('border-color: var(--action-attention)')
    expect(selected).toContain('box-shadow: var(--ring-sel)')
    expect(selected).toContain('color: var(--action-attention)')
  })

  it('keeps the dense rail and independently scrolling content pane', () => {
    expect(rule('.settings-rail').body).toContain('width: 196px')
    expect(rule('.settings-body').body).toContain('overflow: auto')
    expect(rule('.settings-row').body).toContain('border-bottom: 1px solid var(--border-subtle)')
  })
})
