import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Card } from '../../src/ui/Card'
import { read, rules } from '../css'

const ui = read('ui/ui.css')
const rule = (sel: string) => rules(ui).find((r) => r.selector === sel)
const card = (el: HTMLElement) => el.querySelector('div')!

describe('ui/card-selected', () => {
  it('is a plain surface until it is selected', () => {
    const { container } = render(() => <Card>Gate — approve plan</Card>)
    expect(card(container).className).toBe('card')
    expect(card(container).getAttribute('aria-selected')).toBe(null)
  })

  it('marks selection in the DOM as well as in paint', () => {
    const { container } = render(() => <Card selected>Gate — approve plan</Card>)
    expect(card(container).className).toContain('card-selected')
    expect(card(container).getAttribute('aria-selected')).toBe('true')
  })

  it('applies the accent inset ring over the raised surface', () => {
    const body = rule('.card-selected')!.body
    expect(body).toContain('background: var(--sf2)')
    expect(body).toContain('box-shadow: var(--ring-sel)')
    expect(read('styles/tokens.css')).toContain('--ring-sel: inset 0 0 0 1px var(--ac)')
  })

  it('never rings with an outer glow', () => {
    for (const r of rules(ui)) {
      const shadows = r.body.match(/box-shadow:\s*([^;]+)/g) ?? []
      for (const s of shadows) {
        if (!s.includes('var(--ac)') && !s.includes('ring-sel')) continue
        expect(s, `${r.selector}: ${s}`).toMatch(/inset|ring-sel/)
      }
    }
  })

  it('only responds to the pointer when it is told it is interactive', () => {
    const plain = render(() => <Card>read-only</Card>)
    expect(card(plain.container).className).not.toContain('card-interactive')
    const live = render(() => <Card interactive>clickable</Card>)
    expect(card(live.container).className).toContain('card-interactive')
  })
})
