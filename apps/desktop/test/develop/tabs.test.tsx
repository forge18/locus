import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { DevelopView } from '../../src/screens/develop/DevelopView'
import { useEditorTabs } from '../../src/data/develop'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <DevelopView />)

describe('develop/tabs', () => {
  it('is 30px on the deep ground', () => {
    const body = rule('.dev-tabs').body
    expect(body).toContain('height: 30px')
    expect(body).toContain('background: var(--bg-deep)')
    expect(body).toContain('border-bottom: 1px solid var(--line)')
  })

  it('renders one tab per open file', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tabs').querySelectorAll('.dev-tab').length).toBe(useEditorTabs().length)
  })

  it('marks the active tab and paints it --bg with the accent top inset', () => {
    const { getByTestId } = mount()
    expect(getByTestId('dev-tab-notify.rs').getAttribute('aria-selected')).toBe('true')
    const body = rule(".dev-tab[aria-selected='true']").body
    expect(body).toContain('background: var(--bg)')
    expect(body).toContain('box-shadow: inset 0 2px 0 var(--ac)')
  })

  it('gives the active tab a close control, and only it', () => {
    const { getByTestId, queryByTestId } = mount()
    expect(getByTestId('dev-tab-close-notify.rs').textContent).toBe('×')
    expect(queryByTestId('dev-tab-close-mod.rs')).toBe(null)
  })

  it('moves the active tab when another is clicked', () => {
    const { getByTestId } = mount()
    getByTestId('dev-tab-mod.rs').click()
    expect(getByTestId('dev-tab-mod.rs').getAttribute('aria-selected')).toBe('true')
    expect(getByTestId('dev-tab-notify.rs').getAttribute('aria-selected')).toBe('false')
  })

  it('opens a tree file into the tab strip', () => {
    const { getByTestId } = mount()
    getByTestId('dev-tab-mod.rs').click()
    getByTestId('dev-tree-row-crates/locus-core/src/store/notify.rs').click()
    expect(getByTestId('dev-tab-notify.rs').getAttribute('aria-selected')).toBe('true')
  })
})
