import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryWikiFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory wiki viewer', () => {
  it('identifies outline links provenance and graph', () => {
    const { getByTestId } = render(() => <MemoryWikiFixture />)
    expect(getByTestId('desktop-memory-wiki').getAttribute('data-wiki-viewer')).toBe('outline-links-provenance-graph')
  })
})
