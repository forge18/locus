import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Rail } from '../../src/shell/Rail'
import { CATEGORIES, RAIL_ITEMS } from '../../src/nav'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('shell/shell.css')).find((r) => r.selector === sel)
const mount = (onNavigate = () => {}) =>
  render(() => <Rail view="inbox" onNavigate={onNavigate} inboxCount={3} />)

describe('shell/rail-seven-items', () => {
  it('flexes around 78px on the deep ground, with a right hairline', () => {
    const body = rule('.rail')!.body
    expect(body).toContain('width: clamp(68px, 6vw, 92px)')
    expect(body).toContain('background: var(--bg-deep)')
    expect(body).toContain('border-right: 1px solid var(--line)')
    expect(body).toContain('padding: 8px var(--g-3)')
    expect(body).toContain('gap: var(--g-1)')
  })

  it('shows exactly seven items, one per category', () => {
    const { getByTestId } = mount()
    expect(getByTestId('rail').querySelectorAll('.rail-item').length).toBe(7)
    expect(RAIL_ITEMS.length).toBe(CATEGORIES.length)
  })

  it('draws them in the order the handoff states', () => {
    const { getByTestId } = mount()
    expect(
      [...getByTestId('rail').querySelectorAll('.rail-item')].map((el) =>
        el.getAttribute('data-category'),
      ),
    ).toEqual(['dashboard', 'plan', 'develop', 'automate', 'review', 'workshop', 'wiki'])
  })

  it('gives each its Phosphor glyph at 19px over a 9.5px label', () => {
    const { getByTestId } = mount()
    const icons = [...getByTestId('rail').querySelectorAll('.rail-item svg')]
    expect(icons.map((i) => i.getAttribute('width')).slice(0, 7)).toEqual(Array(7).fill('19'))
    expect([...getByTestId('rail').querySelectorAll('.rail-item use')].map((u) => u.getAttribute('href'))).toEqual([
      '#ph-tray', '#ph-compass', '#ph-code', '#ph-lightning', '#ph-chart-bar', '#ph-wrench', '#ph-book-bookmark',
    ])
    expect(rule('.rail-item-label')!.body).toContain('font-size: var(--t-label)')
  })

  it('lands a click on the category first view, not on what was last open there', () => {
    const landed: string[] = []
    const { getByTestId } = mount((v?: unknown) => landed.push(v as string))
    for (const item of RAIL_ITEMS) getByTestId(`rail-${item.category}`).click()
    expect(landed).toEqual(['inbox', 'plan', 'develop', 'board', 'telemetry', 'extensions', 'wiki'])
  })
})
