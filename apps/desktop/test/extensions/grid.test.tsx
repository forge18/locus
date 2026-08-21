import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'
import { useTypeCards } from '../../src/data/extensions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <ExtensionsView onNavigate={() => {}} />)

describe('extensions/grid', () => {
  it('lays eight cards out four to a row where there is room, and reflows where there is not', () => {
    expect(rule('.type-grid').body).toContain('repeat(auto-fit, minmax(200px, 1fr))')
    const { getByTestId } = mount()
    expect(getByTestId('type-grid').querySelectorAll('.type-card').length).toBe(8)
  })

  it('draws each card at radius 9', () => {
    expect(rule('.type-card').body).toContain('border-radius: var(--r-lg)')
    expect(read('styles/tokens.css')).toContain('--r-lg: 9px')
  })

  it('shows an icon and the lowercase type name', () => {
    const { getByTestId } = mount()
    for (const card of useTypeCards()) {
      const el = getByTestId(`type-card-${card.type}`)
      expect(el.querySelector('use'), card.type).not.toBe(null)
      expect(el.textContent, card.type).toContain(card.type)
      expect(card.type).toBe(card.type.toLowerCase())
    }
  })

  it('shows the count at 26px and the description at 14px', () => {
    const { getByTestId } = mount()
    expect(getByTestId('type-count-agents').textContent).toBe('14')
    expect(rule('.type-card-count').body).toContain('font-size: var(--t-metric)')
    expect(rule('.type-card-desc').body).toContain('font-size: var(--t-body)')
  })

  it('gives each type its own glyph', () => {
    const { getByTestId } = mount()
    const icons = useTypeCards().map(
      (c) => getByTestId(`type-card-${c.type}`).querySelector('use')!.getAttribute('href'),
    )
    expect(new Set(icons).size).toBe(icons.length)
  })
})
