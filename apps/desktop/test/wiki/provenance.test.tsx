import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { useWikiPage } from '../../src/data/wiki'

const page = useWikiPage('w-clone')!
const mount = () => render(() => <WikiArticle page={page} onFollow={() => {}} />)

describe('wiki/provenance', () => {
  it('is headed PROVENANCE', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-article').textContent).toContain('Provenance')
  })

  it('lists one row per source', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-provenance').querySelectorAll('.wiki-provenance-row').length).toBe(
      page.provenance.length,
    )
  })

  it('gives each row an icon naming what kind of source it is', () => {
    const { getByTestId } = mount()
    const icons = [...getByTestId('wiki-provenance').querySelectorAll('use')].map((u) =>
      u.getAttribute('href'),
    )
    expect(icons).toEqual(['#ph-file-text', '#ph-git-pull-request'])
  })

  it('says where the page came from and when it was ingested', () => {
    const { getByTestId } = mount()
    const text = getByTestId('wiki-provenance').textContent!
    expect(text).toContain('PLAN.md')
    expect(text).toContain('ingested 4d ago')
    expect(text).toContain('PR #491')
  })

  it('shows no section when a page has no recorded provenance', () => {
    const bare = useWikiPage('w-locusd')!
    const { queryByTestId } = render(() => <WikiArticle page={bare} onFollow={() => {}} />)
    expect(queryByTestId('wiki-provenance')).toBe(null)
  })
})
