import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { CACHE_READ_RATE, DETERMINISM_NOTE } from '../../src/data/extensions'
import { useHarnessSummary } from '../../src/data/harnesses'
import { read, rules } from '../css'

const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)
const summary = useHarnessSummary()

describe('extensions/materialization-card', () => {
  it('is headed MATERIALIZATION with an amber hairline', () => {
    const { getByTestId } = mount()
    expect(getByTestId('materialization').textContent).toContain('Materialization')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.materialization')!.body,
    ).toContain('inset 0 0 0 1px var(--ac-ring)')
  })

  it('states the byte-determinism rule and why it matters', () => {
    const { getByTestId } = mount()
    const note = getByTestId('determinism-note').textContent!
    expect(note).toContain('byte-deterministic')
    expect(note).toContain('no timestamps')
    expect(note).toContain('cache miss')
    expect(DETERMINISM_NOTE).toBe(note)
  })

  it('shows three figures', () => {
    const { getByTestId } = mount()
    expect(getByTestId('materialization').querySelectorAll('.materialization-figure').length).toBe(3)
  })

  it('computes entries and downgrades from the registry', () => {
    const { getByTestId } = mount()
    expect(getByTestId('materialization-entries').textContent).toContain(String(summary.entries))
    expect(getByTestId('materialization-downgrades').textContent).toContain(
      String(summary.downgrades),
    )
    expect(summary.entries).toBe(96)
    expect(summary.downgrades).toBe(33)
  })

  it('shows the cache-read rate, the number determinism exists to protect', () => {
    const { getByTestId } = mount()
    expect(getByTestId('materialization-cache').textContent).toContain(CACHE_READ_RATE)
  })

  it('reports 96 and 33, not the handoff copy’s 88 and 27', () => {
    const { getByTestId } = mount()
    const text = getByTestId('materialization').textContent!
    expect(text).toContain('96')
    expect(text).toContain('33')
    expect(text).not.toContain('88')
    expect(text).not.toContain('27')
  })
})
