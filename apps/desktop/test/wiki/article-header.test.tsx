import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { useWikiPage } from '../../src/data/wiki'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const page = useWikiPage('w-clone')!
const mount = () => render(() => <WikiArticle page={page} onFollow={() => {}} />)

describe('wiki/article-header', () => {
  it('tags the kind in accent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article-kind').textContent).toBe('decision')
    expect(rule('.wiki-kind').body).toContain('color: var(--action-attention)')
    expect(rule('.wiki-kind').body).toContain('background: var(--action-attention-wash)')
  })

  it('sets the title at 19px/500', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article-title').textContent).toBe(page.title)
    expect(rule('.wiki-article-title').body).toContain('font-size: var(--t-title)')
    expect(rule('.wiki-article-title').body).toContain('font-weight: 500')
  })

  it('puts the tag before the title, on one row', () => {
    const { getByTestId } = mount()
    const head = getByTestId('wiki-article').querySelector('.wiki-article-head')!
    expect(head.children[0]).toBe(getByTestId('wiki-article-kind'))
    expect(head.children[1]).toBe(getByTestId('wiki-article-title'))
  })

  it('takes the kind from the page, not from the markup', () => {
    const entity = useWikiPage('w-locusd')!
    const { getByTestId } = render(() => <WikiArticle page={entity} onFollow={() => {}} />)
    expect(getByTestId('wiki-article-kind').textContent).toBe('entity')
  })
})
