import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useExtensionTypes, useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <HarnessesView />)
const types = useExtensionTypes()

describe('harnesses/capability-bar', () => {
  it('has exactly eight segments on every card, one per extension type', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      expect(
        getByTestId(`hn-bar-${harness.name}`).querySelectorAll('.hn-seg').length,
        harness.name,
      ).toBe(8)
    }
    expect(types.length).toBe(8)
  })

  it('colours each segment from that harness’s own TOML entry', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      for (const type of types) {
        const entry = harness.extensions.find((e) => e.type === type)!
        const seg = getByTestId(`hn-seg-${harness.name}-${type}`)
        expect(seg.getAttribute('data-native'), `${harness.name}/${type}`).toBe(
          entry.weakerThanNative === null ? 'true' : 'false',
        )
        expect(seg.className, `${harness.name}/${type}`).toContain(
          entry.weakerThanNative === null ? 'hn-seg-native' : 'hn-seg-downgraded',
        )
      }
    }
  })

  it('paints native accent and downgraded red', () => {
    expect(rule('.hn-seg-native').body).toContain('background: var(--ac)')
    expect(rule('.hn-seg-downgraded').body).toContain('rgba(212,97,79,.55)')
  })

  it('names the loss on every downgraded segment', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      for (const entry of harness.extensions) {
        if (!entry.weakerThanNative) continue
        expect(
          getByTestId(`hn-seg-${harness.name}-${entry.type}`).getAttribute('title'),
          `${harness.name}/${entry.type}`,
        ).toContain(entry.weakerThanNative)
      }
    }
  })

  it('shows all eight native on the reference harnesses', () => {
    const { getByTestId } = mount()
    for (const name of ['claude', 'pi', 'omp']) {
      const segs = [...getByTestId(`hn-bar-${name}`).querySelectorAll('.hn-seg')]
      expect(segs.every((s) => s.getAttribute('data-native') === 'true'), name).toBe(true)
    }
  })

  it('states the extension count from the registry, not from a literal', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hn-extension-count-claude').textContent).toBe('8 extensions')
  })
})
