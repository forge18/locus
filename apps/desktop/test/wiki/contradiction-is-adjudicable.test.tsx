import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiSidebar } from '../../src/screens/wiki/WikiSidebar'
import { useContradictions } from '../../src/data/wiki'
import type { Contradiction } from '../../src/types/wiki'

describe('wiki/contradiction-is-adjudicable', () => {
  it('carries a source on every side, in every fixture', () => {
    for (const c of useContradictions()) {
      expect(c.values.length, c.id).toBe(2)
      for (const side of c.values) {
        expect(side.source.length, c.id).toBeGreaterThan(0)
        expect(side.value.length, c.id).toBeGreaterThan(0)
      }
    }
  })

  it('types both sides as required — a one-sided flag cannot be constructed', () => {
    const oneSided: Contradiction = {
      id: 'x', pageIds: ['a', 'b'], claim: 'c', detectedAt: '', resolved: false,
      // @ts-expect-error — values is a pair, not a list that can be short
      values: [{ value: 'v', source: 's', age: '1d' }],
      note: 'n',
    }
    void oneSided
  })

  it('types the source as required on each side', () => {
    // @ts-expect-error — source is not optional
    const noSource: Contradiction['values'][0] = { value: 'v', age: '1d' }
    void noSource
  })

  it('renders both sources on screen, so the reader can adjudicate from what is there', () => {
    const { getByTestId } = render(() => <WikiSidebar />)
    const card = getByTestId('contradiction-x-1')
    const sources = [...card.querySelectorAll('.contradiction-source')].map((s) => s.textContent)
    expect(sources.length).toBe(2)
    expect(new Set(sources).size).toBe(2)
  })

  it('offers an action for each way it can be settled — decide, or defer to the board', () => {
    const { getByTestId } = render(() => <WikiSidebar />)
    expect(getByTestId('contradiction-adjudicate')).toBeTruthy()
    expect(getByTestId('contradiction-board-card')).toBeTruthy()
  })
})
