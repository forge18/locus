import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WikiView } from '../../src/screens/wiki/WikiView'
import { createNavStore } from '../../src/nav'
import { useWikiLint, useWikiPages } from '../../src/data/wiki'
import { read } from '../css'

const mount = () => render(() => <WikiView nav={createNavStore({ view: 'wiki' })} />)

describe('wiki/orphan-single-source', () => {
  it('counts the same orphans in the tree and in the lint card', () => {
    const { getByTestId, container } = mount()
    const flagged = container.querySelectorAll('.wiki-orphan').length
    const counted = useWikiLint().find((f) => f.kind === 'orphan')!.count
    expect(counted).toBe(flagged)
    expect(getByTestId('lint-orphan').textContent).toContain(String(flagged))
  })

  it('derives the lint count from the pages rather than writing it down', () => {
    const source = read('fixtures/wiki.ts')
    expect(source).toContain('count: PAGES.filter((p) => p.orphan).length')
  })

  it('names the same pages in both places', () => {
    const { container, getByTestId } = mount()
    const titles = useWikiPages().filter((p) => p.orphan).map((p) => p.title)
    for (const title of titles) {
      expect(getByTestId('lint-orphan').textContent, title).toContain(title)
    }
    expect(container.querySelectorAll('.wiki-orphan').length).toBe(titles.length)
  })

  it('moves both together — one condition, two surfaces, one source', () => {
    const orphans = useWikiPages().filter((p) => p.orphan)
    const counted = useWikiLint().find((f) => f.kind === 'orphan')!
    expect(counted.count).toBe(orphans.length)
    expect(counted.detail).toBe(orphans.map((p) => p.title).join(', '))
  })
})
