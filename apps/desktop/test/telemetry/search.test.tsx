import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { SEARCH_NOTE, SEARCH_QUERY } from '../../src/data/telemetry'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)

describe('telemetry/search', () => {
  it('shows the query in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-query').textContent).toBe(SEARCH_QUERY)
    expect(rule('.tm-query').body).toContain('font-family: var(--fm)')
  })

  it('blinks an accent caret after it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-caret').className).toContain('blink')
    expect(rule('.tm-caret').body).toContain('background: var(--action-attention)')
  })

  it('says what is being searched, and how', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-search-note').textContent).toBe(SEARCH_NOTE)
    expect(SEARCH_NOTE).toContain('every event, every session')
    expect(SEARCH_NOTE).toContain('BM25 over the normalized log')
  })

  it('leads with a magnifier', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-search').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-magnifying-glass',
    )
  })

  it('grounds the bar on --sf', () => {
    expect(rule('.tm-search').body).toContain('background: var(--surface-raised)')
  })
})
