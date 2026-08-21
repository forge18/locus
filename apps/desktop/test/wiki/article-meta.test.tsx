import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { parse } from '../../src/nav'
import { useWikiPage } from '../../src/data/wiki'

const page = useWikiPage('w-clone')!
const mount = () => render(() => <WikiArticle page={page} onFollow={() => {}} />)

describe('wiki/article-meta', () => {
  it('shows a mono locator that actually parses', () => {
    const { getByTestId } = mount()
    const locator = getByTestId('wiki-article-locator')
    expect(locator.className).toContain('mono')
    expect(() => parse(locator.textContent!)).not.toThrow()
    expect(parse(locator.textContent!).kind).toBe('page')
  })

  it('shows the revision, in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article-rev').textContent).toBe('rev 7')
    expect(getByTestId('wiki-article-rev').className).toContain('mono')
  })

  it('shows the assertion and source counts', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article-counts').textContent).toBe('3 assertions · 2 sources')
  })

  it('shows when it was ingested and when it was curated — two different things', () => {
    const { getByTestId } = mount()
    const ages = getByTestId('wiki-article-ages').textContent!
    expect(ages).toContain('Ingest 4d ago')
    expect(ages).toContain('curated 2d ago')
  })

  it('takes all four from the page rather than the markup', () => {
    const other = useWikiPage('w-locusd')!
    const { getByTestId } = render(() => <WikiArticle page={other} onFollow={() => {}} />)
    expect(getByTestId('wiki-article-rev').textContent).toBe('rev 1')
    expect(getByTestId('wiki-article-counts').textContent).toBe('0 assertions · 0 sources')
  })
})
