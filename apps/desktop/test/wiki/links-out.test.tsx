import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { useWikiPage } from '../../src/data/wiki'
import { read, rules } from '../css'

const page = useWikiPage('w-clone')!
const mount = () => render(() => <WikiArticle page={page} onFollow={() => {}} />)

describe('wiki/links-out', () => {
  it('is headed LINKS OUT', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article').textContent).toContain('Links out')
  })

  it('renders one pill per link', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-links').querySelectorAll('.wikilink').length).toBe(page.links.length)
  })

  it('renders them as [[wikilinks]], in mono', () => {
    const { getByTestId } = mount()
    const first = getByTestId('wiki-links').querySelector('.wikilink')!
    expect(first.textContent).toBe(`[[${page.links[0]}]]`)
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.wikilink')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('pills them on --sf with a hairline', () => {
    const body = rules(read('screens/screens.css')).find((r) => r.selector === '.wikilink')!.body
    expect(body).toContain('background: var(--sf)')
    expect(body).toContain('border: 1px solid var(--line)')
  })

  it('shows no section at all when a page links nowhere', () => {
    const bare = useWikiPage('w-locusd')!
    const { queryByTestId } = render(() => <WikiArticle page={bare} onFollow={() => {}} />)
    expect(queryByTestId('wiki-links')).toBe(null)
  })
})
