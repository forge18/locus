import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryWikiFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory wiki route', () => {
  it('identifies the typed wiki fixture route', () => {
    const { getByTestId } = render(() => <MemoryWikiFixture />)
    expect(getByTestId('desktop-memory-wiki').getAttribute('data-wiki-fixture')).toBe('typed-page')
  })
})
