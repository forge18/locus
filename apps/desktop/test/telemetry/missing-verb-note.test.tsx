import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { MISSING_VERB_NOTE } from '../../src/data/telemetry'

const mount = () => render(() => <TelemetryView />)

describe('telemetry/missing-verb-note', () => {
  it('renders beside the action list', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-actions').contains(getByTestId('missing-verb-note'))).toBe(true)
  })

  it('says a missing verb is recorded as missing, never synthesized', () => {
    const { getByTestId } = mount()
    expect(getByTestId('missing-verb-note').textContent).toContain(
      'recorded as missing, never synthesized',
    )
  })

  it('names which verb is absent and for which runs', () => {
    expect(MISSING_VERB_NOTE).toContain('thinking')
    expect(MISSING_VERB_NOTE).toContain('never reported it')
  })

  it('sits after the rows, not among them', () => {
    const { getByTestId } = mount()
    const panel = getByTestId('tm-actions')
    expect(panel.children[panel.children.length - 1]).toBe(getByTestId('missing-verb-note'))
  })
})
