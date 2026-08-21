import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { TIERS, resolveTier } from '../../src/data/settings'
import { useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const mount = () => render(() => <HarnessesView />)

describe('harnesses/tier-grid', () => {
  it('shows four rows on every card', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      expect(
        getByTestId(`hn-tiers-${harness.name}`).querySelectorAll('.hn-tier').length,
        harness.name,
      ).toBe(4)
    }
    expect(TIERS).toEqual(['low', 'medium', 'high', 'xhigh'])
  })

  it('labels them low, med, high, xhigh', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-tier-claude-low').textContent).toContain('low')
    // `medium` is drawn as `med`, so the four labels fit one column.
    expect(getByTestId('hn-tier-claude-medium').textContent).toContain('med')
    expect(getByTestId('hn-tier-claude-medium').textContent).not.toContain('medium')
    expect(getByTestId('hn-tier-claude-high').textContent).toContain('high')
    expect(getByTestId('hn-tier-claude-xhigh').textContent).toContain('xhigh')
  })

  it('shows each mapped model in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-tier-claude-low').textContent).toContain('haiku-4.5')
    expect(getByTestId('hn-tier-claude-high').textContent).toContain('opus-4.6')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.hn-tier-value')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('accents the high row', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-tier-claude-high').className).toContain('hn-tier-high')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.hn-tier-high .hn-tier-value')!
        .body,
    ).toContain('color: var(--ac)')
  })

  it('reads the mapping from settings, which is policy rather than mechanism', () => {
    expect(resolveTier('claude', 'medium').model).toBe('sonnet-4.6')
    expect(resolveTier('codex', 'medium').model).toBe('gpt-5.2')
  })

  it('has a row for every tier on every harness, mapped or not', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      for (const tier of TIERS) {
        expect(getByTestId(`hn-tier-${harness.name}-${tier}`), `${harness.name}/${tier}`).toBeTruthy()
      }
    }
  })
})
