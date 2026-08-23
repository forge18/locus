import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { useFacetGroups } from '../../src/data/telemetry'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)
const groups = useFacetGroups()

describe('telemetry/facets', () => {
  it('shows the eight groups the design draws', () => {
    const { getByTestId } = mount()
    expect(groups.map((g) => g.key)).toEqual([
      'harness',
      'capture_source',
      'project',
      'agent_role',
      'model_tier',
      'verify',
      'arbiter_class',
      'branch',
    ])
    for (const group of groups) {
      expect(getByTestId(`facet-group-${group.key}`), group.key).toBeTruthy()
    }
  })

  it('shows a count on every chip', () => {
    const { getByTestId } = mount()
    for (const group of groups) {
      for (const facet of group.facets) {
        const chip = getByTestId(`facet-${group.key}-${facet.value.replace(/\W+/g, '-')}`)
        expect(chip.querySelector('.facet-count')!.textContent, facet.value).toBe(
          String(facet.count),
        )
      }
    }
  })

  it('grounds the chips on --sf3 with the count in --mu2', () => {
    expect(rule('.facet').body).toContain('background: var(--surface-elevated)')
    expect(rule('.facet-count').body).toContain('color: var(--text-muted)')
  })

  it('sets the counts in mono', () => {
    expect(rule('.facet-count').body).toContain('font-family: var(--fm)')
  })

  it('labels every group', () => {
    const { getByTestId } = mount()
    for (const group of groups) {
      expect(getByTestId(`facet-group-${group.key}`).textContent, group.key).toContain(group.label)
    }
  })

  it('counts the capture source over the one PLAN.md name', () => {
    // ACP is the only harness interface, so there is one source. `hooks`,
    // `stream-json`, and `session-log` are retired — PLAN.md §ACP.
    const capture = groups.find((g) => g.key === 'capture_source')!
    expect(capture.facets.map((f) => f.value)).toEqual(['acp'])
  })
})
