import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <HarnessesView />)

describe('harnesses/mechanism-badges', () => {
  it('badges every card', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      expect(getByTestId(`hn-badge-${harness.name}`).textContent, harness.name).toBe(
        harness.badge.label,
      )
    }
  })

  it('has three variants and no more', () => {
    const variants = new Set(useHarnesses().map((h) => h.badge.variant))
    expect([...variants].sort()).toEqual(['acp', 'bridged', 'native'])
  })

  it('gives the native hooks path the accent tint', () => {
    const { getByTestId } = mount()
    const native = useHarnesses().find((h) => h.badge.variant === 'native')!
    expect(getByTestId(`hn-badge-${native.name}`).className).toContain('hn-badge-native')
    expect(rule('.hn-badge-native').body).toContain('background: var(--ac-wash)')
    expect(native.badge.label).toBe('hooks')
  })

  it('puts a bridged path on --sf3 and names what the bridge is', () => {
    const { getByTestId } = mount()
    const bridged = useHarnesses().filter((h) => h.badge.variant === 'bridged')
    expect(bridged.length).toBeGreaterThan(0)
    expect(getByTestId(`hn-badge-${bridged[0].name}`).className).toContain('hn-badge-bridged')
    expect(rule('.hn-badge-bridged').body).toContain('background: var(--sf3)')
    expect(useHarnesses().find((h) => h.name === 'hermes')!.badge.label).toBe('hooks · plugin')
  })

  it('gives ACP its own blue', () => {
    const { getByTestId } = mount()
    const acp = useHarnesses().find((h) => h.badge.variant === 'acp')!
    expect(acp.name).toBe('cursor')
    expect(getByTestId(`hn-badge-${acp.name}`).textContent).toBe('ACP')
    expect(rule('.hn-badge-acp').body).toContain('rgba(143,184,214,.18)')
    expect(rule('.hn-badge-acp').body).toContain('color: var(--code-keyword)')
  })

  it('derives the badge from the file, never from a table in the source', () => {
    expect(read('screens/workshop/HarnessesView.tsx')).toContain('harness.badge.label')
    expect(read('screens/workshop/HarnessesView.tsx')).not.toMatch(/'hooks · plugin'|'ACP'/)
  })
})
