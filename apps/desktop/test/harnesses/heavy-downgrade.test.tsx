import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <HarnessesView />)
const downgradesOf = (name: string) =>
  useHarnesses().find((h) => h.name === name)!.extensions.filter((e) => e.weakerThanNative).length

describe('harnesses/heavy-downgrade', () => {
  it('hairlines a card in red at four downgrades or more', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      const heavy = downgradesOf(harness.name) >= 4
      expect(getByTestId(`hn-card-${harness.name}`).getAttribute('data-heavy'), harness.name).toBe(
        heavy ? 'true' : null,
      )
    }
    expect(rule('.hn-card-heavy').body).toContain('border-color: var(--bad)')
  })

  it('marks aider, which is the weakest of the set', () => {
    const { getByTestId } = mount()
    expect(downgradesOf('aider')).toBe(6)
    expect(getByTestId('hn-card-aider').getAttribute('data-heavy')).toBe('true')
  })

  it('leaves a lightly downgraded card on the ordinary hairline', () => {
    const { getByTestId } = mount()
    expect(downgradesOf('codex')).toBe(2)
    expect(getByTestId('hn-card-codex').getAttribute('data-heavy')).toBe(null)
  })

  it('sets the downgrade count in --bad at four or more', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-downgrades-aider').className).toContain('hn-downgrades-bad')
    expect(getByTestId('hn-downgrades-codex').className).not.toContain('hn-downgrades-bad')
    expect(rule('.hn-downgrades-bad').body).toContain('color: var(--bad)')
  })

  it('reads "all native" where nothing was lost', () => {
    const { getByTestId } = mount()
    for (const name of ['claude', 'pi', 'omp']) {
      expect(getByTestId(`hn-downgrades-${name}`).textContent, name).toBe('all native')
    }
  })

  it('counts each card’s downgrades from its own entries', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      const n = downgradesOf(harness.name)
      if (n === 0) continue
      expect(getByTestId(`hn-downgrades-${harness.name}`).textContent, harness.name).toBe(
        `${n} downgraded`,
      )
    }
  })
})
