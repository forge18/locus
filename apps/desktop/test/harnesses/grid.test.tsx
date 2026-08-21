import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { HarnessesView } from '../../src/screens/workshop/HarnessesView'
import { useHarnesses } from '../../src/data/harnesses'
import { read, rules } from '../css'

const mount = () => render(() => <HarnessesView />)

describe('harnesses/grid', () => {
  it('is four to a row where there is room, and reflows where there is not', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.hn-grid')!.body,
    ).toContain('repeat(auto-fit, minmax(230px, 1fr))')
  })

  it('draws one card per registered harness', () => {
    const { getByTestId } = mount()
    expect(getByTestId('harnesses-grid').querySelectorAll('.hn-card').length).toBe(
      useHarnesses().length,
    )
  })

  it('names each by the name inside its file, with its binary in mono', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      const card = getByTestId(`hn-card-${harness.name}`)
      expect(card.querySelector('.hn-name')!.textContent, harness.name).toBe(harness.name)
      expect(card.querySelector('.hn-id')!.textContent, harness.name).toBe(harness.binary)
    }
  })

  it('states the injection line on every card', () => {
    const { getByTestId } = mount()
    for (const harness of useHarnesses()) {
      expect(getByTestId(`hn-injection-${harness.name}`).textContent, harness.name).toBe(
        `injection: ${harness.injection}`,
      )
    }
  })

  it('draws the cards in registry order, which is sorted', () => {
    const { getByTestId } = mount()
    const names = [...getByTestId('harnesses-grid').querySelectorAll('.hn-name')].map(
      (n) => n.textContent,
    )
    expect(names).toEqual([...names].sort())
  })
})
