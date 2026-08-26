import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WIKI_KIND_CHIPS } from '../../src/data/knowledge'
import { MemoryWikiFixture } from '../../src/screens/memory/MemoryFixtures'

describe('wiki/from-core', () => {
  it('renders the typed page, visible kinds, provenance, and ingest entry point', () => {
    const { getByTestId, getByText } = render(() => <MemoryWikiFixture />)
    const wiki = getByTestId('desktop-memory-wiki')
    expect(getByText('Ingest a document')).toBeTruthy()
    expect(getByText('Provenance')).toBeTruthy()
    expect(wiki.querySelectorAll('[data-kind]').length).toBe(WIKI_KIND_CHIPS.length)
    expect(wiki.querySelector('[data-kind="overview"]')).toBeNull()
  })
})
