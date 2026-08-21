import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { EVENT_VERBS } from '../../src/types/event'
import { useActionRows } from '../../src/data/telemetry'

const mount = () => render(() => <TelemetryView />)

describe('telemetry/twelve-verbs', () => {
  it('lists exactly twelve actions', () => {
    expect(useActionRows().length).toBe(12)
  })

  it('lists exactly the canonical twelve, no more and no fewer', () => {
    expect(useActionRows().map((a) => a.verb).sort()).toEqual([...EVENT_VERBS].sort())
  })

  it('renders a row for every one of them', () => {
    const { getByTestId } = mount()
    for (const verb of EVENT_VERBS) {
      expect(getByTestId(`action-${verb}`), verb).toBeTruthy()
    }
  })

  it('renders no row for anything else', () => {
    const { getByTestId } = mount()
    const verbs = [...getByTestId('tm-actions').querySelectorAll('.bar-row')].map(
      (r) => r.getAttribute('data-testid')?.replace('action-', ''),
    )
    for (const verb of verbs) expect(EVENT_VERBS, verb).toContain(verb as never)
  })

  it('orders them by count, busiest first', () => {
    const counts = useActionRows().map((a) => a.count)
    expect(counts).toEqual([...counts].sort((a, b) => b - a))
  })
})
