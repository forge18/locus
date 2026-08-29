import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { SEARCH_NOTE, SEARCH_QUERY } from '../../src/data/telemetry'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)

describe('telemetry/search', () => {
  it('is a real input, seeded empty with the fixture query as its placeholder', () => {
    const { getByTestId } = mount()
    const input = getByTestId('tm-query')
    expect(input.tagName).toBe('INPUT')
    expect(input.getAttribute('type')).toBe('search')
    expect((input as HTMLInputElement).value).toBe('')
    expect(input.getAttribute('placeholder')).toBe(SEARCH_QUERY)
  })

  it('wears the mono query style', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-query').className).toBe('tm-query')
    expect(rule('.tm-query').body).toContain('font-family: var(--fm)')
  })

  it('draws no fake caret — the input owns one', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-search').querySelector('.tm-caret')).toBeNull()
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
