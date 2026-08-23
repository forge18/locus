import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryLongTermFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory long-term route', () => {
  it('identifies the long-term fact fixture', () => {
    const { getByTestId } = render(() => <MemoryLongTermFixture />)
    expect(getByTestId('v2-memory-long-term').getAttribute('data-fact-fixture')).toBe('long-term')
  })
})
