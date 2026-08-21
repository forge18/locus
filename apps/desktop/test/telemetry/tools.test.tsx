import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { TOOL_ANOMALY, TOOL_NOTE, useToolRows } from '../../src/data/telemetry'
import { read, rules } from '../css'

const mount = () => render(() => <TelemetryView />)

describe('telemetry/tools', () => {
  it('draws one row per tool', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-tools').querySelectorAll('.bar-row').length).toBe(useToolRows().length)
  })

  it('gives the tool label a 112px column, narrower than the verb one', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.bar-label-tool',
    )!.body
    expect(body).toContain('width: 112px')
    const { getByTestId } = mount()
    expect(getByTestId('tool-bash').querySelector('.bar-label')!.className).toContain(
      'bar-label-tool',
    )
  })

  it('says the counts come from the arbiter', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-tools').textContent).toContain(TOOL_NOTE)
  })

  it('names the anomaly and what it is not', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tool-anomaly').textContent).toBe(TOOL_ANOMALY)
    expect(TOOL_ANOMALY).toContain('already a query, not new instrumentation')
  })

  it('counts the locus CLI verbs among the tools, because they are tools', () => {
    const tools = useToolRows().map((t) => t.tool)
    expect(tools).toContain('locus memory')
    expect(tools).toContain('locus ask')
  })

  it('orders them busiest first', () => {
    const counts = useToolRows().map((t) => t.count)
    expect(counts).toEqual([...counts].sort((a, b) => b - a))
  })
})
