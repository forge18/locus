import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiView } from '../../src/screens/wiki/WikiView'
import { WikiArticle } from '../../src/screens/wiki/WikiArticle'
import { createNavStore, parse } from '../../src/nav'
import { useWikiPage } from '../../src/data/wiki'

const page = useWikiPage('w-clone')!

const mountView = () => {
  const nav = createNavStore({ view: 'wiki' })
  const r = render(() => <WikiView nav={nav} />)
  return { nav, ...r }
}

describe('wiki/wikilink-navigates', () => {
  it('reports the slug when a pill is followed', () => {
    let followed: string | null = null
    const { getByTestId } = render(() => (
      <WikiArticle page={page} onFollow={(slug) => (followed = slug)} />
    ))
    getByTestId('wikilink-bare-local-remote').click()
    expect(followed).toBe('bare local remote')
  })

  it('navigates by locator, through the resolver', () => {
    const { nav, getByTestId } = mountView()
    const before = nav.history().length
    getByTestId('wikilink-bare-local-remote').click()
    expect(nav.view()).toBe('wiki')
    expect(nav.params().slug).toBe('bare-local-remote')
    expect(nav.history().length).toBe(before + 1)
  })

  it('produces a locator that parses as a page', () => {
    const { nav, getByTestId } = mountView()
    getByTestId('wikilink-locus-agent-credential').click()
    expect(parse(nav.locator()).kind).toBe('page')
    expect(nav.locator()).toBe('locus://tapestry/page/locus-agent-credential')
  })

  it('stays in the Wiki category — a wikilink is not a category change', () => {
    const { nav, getByTestId } = mountView()
    getByTestId('wikilink-Sculptor').click()
    expect(nav.category()).toBe('wiki')
  })

  it('is reversible, because the locator went on the history stack', () => {
    const { nav, getByTestId } = mountView()
    getByTestId('wikilink-bare-local-remote').click()
    nav.back()
    expect(nav.params().slug).toBe(undefined)
  })
})
