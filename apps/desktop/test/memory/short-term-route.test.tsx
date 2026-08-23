import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryShortTermFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory short-term route', () => {
  it('identifies the short-term context fixture route', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(getByTestId('desktop-memory-short-term').getAttribute('data-desktop-route')).toBe('memory-short-term')
  })
})
