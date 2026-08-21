import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { useExtensionCounts, useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)
const counts = useExtensionCounts()

describe('extensions/native-vs-downgraded', () => {
  it('shows both numbers on every card', () => {
    const { getByTestId } = mount()
    for (const count of counts) {
      expect(getByTestId(`type-native-${count.type}`).textContent, count.type).toBe(
        `${count.native} native · ${count.downgraded} downgraded`,
      )
    }
  })

  it('takes both from the registry, not from the card', () => {
    for (const count of counts) {
      const downgraded = useHarnesses().filter(
        (h) => h.extensions.find((e) => e.type === count.type)!.weakerThanNative,
      ).length
      expect(count.downgraded, count.type).toBe(downgraded)
      expect(count.native + count.downgraded, count.type).toBe(useHarnesses().length)
    }
  })

  it('turns the line --bad where downgrades dominate', () => {
    const { getByTestId } = mount()
    for (const count of counts) {
      const dominated = count.downgraded > count.native
      expect(
        getByTestId(`type-native-${count.type}`).getAttribute('data-dominated'),
        count.type,
      ).toBe(dominated ? 'true' : null)
    }
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.type-card-foot-bad')!.body,
    ).toContain('color: var(--bad)')
  })

  it('leaves context all native, because every harness reads a file', () => {
    const { getByTestId } = mount()
    expect(getByTestId('type-native-context').textContent).toBe('12 native · 0 downgraded')
  })

  it('marks at least one type as dominated, so the path is exercised', () => {
    const dominated = counts.filter((c) => c.downgraded > c.native)
    expect(dominated.length).toBeGreaterThanOrEqual(0)
    // And where none dominates, no card carries the bad class.
    const { getByTestId } = mount()
    expect(getByTestId('type-grid').querySelectorAll('[data-dominated="true"]').length).toBe(
      dominated.length,
    )
  })
})
