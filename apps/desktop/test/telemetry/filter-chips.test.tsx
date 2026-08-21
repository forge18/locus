import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { RESET_LABEL, useFilterChips } from '../../src/data/telemetry'
import { read, rules } from '../css'

const mount = () => render(() => <TelemetryView />)

describe('telemetry/filter-chips', () => {
  it('shows one chip per active filter', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-chips').querySelectorAll('.tag').length).toBe(useFilterChips().length)
  })

  it('names the filters the design draws', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-chip-verify-failed').textContent).toBe('verify: failed')
    expect(getByTestId('tm-chip-30d').textContent).toBe('30d')
  })

  it('draws them as outline chips — accent as a line', () => {
    const { getByTestId } = mount()
    for (const chip of getByTestId('tm-chips').querySelectorAll('.tag')) {
      expect(chip.className).toContain('tag-outline')
    }
    expect(
      rules(read('ui/ui.css')).find((r) => r.selector === '.tag-outline')!.body,
    ).toContain('border-color: var(--ac)')
  })

  it('offers a Reset control in accent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-reset').textContent).toBe(RESET_LABEL)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.tm-reset')!.body,
    ).toContain('color: var(--ac)')
  })

  it('marks which chips are active in the DOM', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-chip-30d').getAttribute('data-active')).toBe('true')
  })
})
