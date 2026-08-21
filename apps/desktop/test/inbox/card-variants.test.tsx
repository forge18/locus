import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxCard } from '../../src/screens/inbox/InboxCard'
import { useInboxItems } from '../../src/data/inbox'
import { read, rules } from '../css'

const [gate, ask, guardrail] = useInboxItems()
const mount = (item = gate, selected = false) =>
  render(() => <InboxCard item={item} selected={selected} onSelect={() => {}} />)
const iconOf = (el: HTMLElement) => el.querySelector('use')!.getAttribute('href')

describe('inbox/card-variants', () => {
  it('gives a gate the filled seal-check', () => {
    const { getByTestId } = mount(gate)
    expect(iconOf(getByTestId(`inbox-card-${gate.id}`))).toBe('#ph-seal-check-fill')
  })

  it('gives an ask the question mark', () => {
    const { getByTestId } = mount(ask)
    expect(iconOf(getByTestId(`inbox-card-${ask.id}`))).toBe('#ph-question')
    expect(getByTestId(`inbox-card-${ask.id}`).textContent).toContain('locus ask')
  })

  it('gives a guardrail the filled warning-octagon in --bad', () => {
    const { getByTestId } = mount(guardrail)
    const card = getByTestId(`inbox-card-${guardrail.id}`)
    expect(iconOf(card)).toBe('#ph-warning-octagon-fill')
    expect(card.querySelector('svg')!.getAttribute('style')).toContain('var(--bad)')
  })

  it('marks the selected card in the DOM and paints it --sf2 with the accent ring', () => {
    const { getByTestId } = mount(gate, true)
    expect(getByTestId(`inbox-card-${gate.id}`).getAttribute('aria-selected')).toBe('true')
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === ".inbox-card[aria-selected='true']",
    )!
    expect(rule.body).toContain('background: var(--sf2)')
    expect(rule.body).toContain('box-shadow: var(--ring-sel)')
  })

  it('leaves an unselected card on --sf with a hairline', () => {
    const { getByTestId } = mount(gate, false)
    expect(getByTestId(`inbox-card-${gate.id}`).getAttribute('aria-selected')).toBe('false')
    const rule = rules(read('screens/screens.css')).find((r) => r.selector === '.inbox-card')!
    expect(rule.body).toContain('background: var(--sf)')
    expect(rule.body).toContain('border: 1px solid var(--line)')
  })
})
