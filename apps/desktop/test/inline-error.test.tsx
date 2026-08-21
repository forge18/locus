import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InlineError } from '../src/ui/InlineError'

describe('inline-error', () => {
  const mount = () =>
    render(() => (
      <InlineError
        cause="Materialization failed for pi: harnesses/pi.toml is unreadable"
        next="Fix the file, then re-run the materializer"
      />
    ))

  it('carries what failed', () => {
    expect(mount().getByTestId('inline-error-cause').textContent).toContain(
      'harnesses/pi.toml is unreadable',
    )
  })

  it('carries what to do about it', () => {
    expect(mount().getByTestId('inline-error-next').textContent).toContain('re-run the materializer')
  })

  it('renders in --bad, on the surface that failed', () => {
    const el = mount().getByTestId('inline-error') as HTMLElement
    expect(el.style.color).toBe('var(--bad)')
    expect(el.style.boxShadow).toContain('var(--bad)')
  })

  it('announces itself where it sits, rather than as a toast', () => {
    expect(mount().getByTestId('inline-error').getAttribute('role')).toBe('alert')
  })

  it('takes an action control when the next step is a click', () => {
    const { getByText } = render(() => (
      <InlineError cause="Push rejected" next="Rebase on main" action={<button>Rebase</button>} />
    ))
    expect(getByText('Rebase')).toBeTruthy()
  })
})
