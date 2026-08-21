import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiView } from '../../src/screens/wiki/WikiView'
import { createNavStore } from '../../src/nav'
import { useDefaultPageId } from '../../src/data/wiki'
import { read, rules } from '../css'

const mount = () => render(() => <WikiView nav={createNavStore({ view: 'wiki' })} />)

describe('wiki/selected-page', () => {
  it('marks exactly one page selected', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('wiki-tree').querySelectorAll('[aria-selected="true"]')
    expect(marked.length).toBe(1)
    expect(marked[0].getAttribute('data-testid')).toBe(`wiki-page-${useDefaultPageId()}`)
  })

  it('paints it --sf2 with the accent inset ring', () => {
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === ".wiki-page[aria-selected='true']",
    )!
    expect(rule.body).toContain('background: var(--sf2)')
    expect(rule.body).toContain('box-shadow: var(--ring-sel)')
  })

  it('shows the selected page in the article pane', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article-title').textContent).toBe(
      'Clone from a local bare remote, never a mount',
    )
  })

  it('moves the article when another page is picked', () => {
    const { getByTestId } = mount()
    getByTestId('wiki-page-w-locusd').click()
    expect(getByTestId('wiki-article-title').textContent).toBe('locusd')
    expect(getByTestId('wiki-page-w-locusd').getAttribute('aria-selected')).toBe('true')
    expect(getByTestId('wiki-page-w-clone').getAttribute('aria-selected')).toBe('false')
  })
})
