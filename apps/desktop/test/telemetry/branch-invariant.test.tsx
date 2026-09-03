import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { useFacetGroups } from '../../src/data/telemetry'
import { read, rules } from '../css'

const mount = () => render(() => <TelemetryView />)
const branch = useFacetGroups().find((g) => g.key === 'branch')!

describe('telemetry/branch-invariant', () => {
  it('shows main with a count of zero', () => {
    const { getByTestId } = mount()
    const chip = getByTestId('facet-branch-main')
    expect(chip.textContent).toContain('main')
    expect(chip.querySelector('.facet-count')!.textContent).toBe('0')
  })

  it('uses muted semantic color because the zero is by design', () => {
    const { getByTestId } = mount()
    expect(getByTestId('facet-branch-main').className).toContain('facet-invariant')
    const body = rules(read('screens/screens.css')).find((r) => r.selector === '.facet-invariant')!.body
    expect(body).toContain('color: var(--text-muted)')
    expect(body).toContain('background: var(--surface-ground)')
  })

  it('shows every other branch under agent/*', () => {
    const { getByTestId } = mount()
    expect(getByTestId('facet-branch-agent-').querySelector('.facet-count')!.textContent).toBe('641')
  })

  it('marks the invariant in the data, not only in the paint', () => {
    const main = branch.facets.find((f) => f.value === 'main')!
    expect(main.count).toBe(0)
    expect(main.invariant).toBe(true)
  })

  it('proves the invariant rather than claiming it — the facet is the evidence', () => {
    const total = branch.facets.reduce((n, f) => n + f.count, 0)
    expect(branch.facets.find((f) => f.value === 'agent/*')!.count).toBe(total)
  })
})
