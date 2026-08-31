import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryLongTermFixture } from '../../src/demo/MemoryFixtures'

describe('Memory long-term route', () => {
  it('identifies the long-term fact fixture', () => {
    const { getByTestId } = render(() => <MemoryLongTermFixture />)
    expect(getByTestId('desktop-memory-long-term').getAttribute('data-fact-fixture')).toBe('long-term')
  })
})
