import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryLongTermFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory fact provenance', () => {
  it('identifies provenance, confidence, decay, and contradiction state', () => {
    const { getByTestId } = render(() => <MemoryLongTermFixture />)
    expect(getByTestId('desktop-memory-long-term').getAttribute('data-fact-state')).toBe('provenance-confidence-decay-contradiction')
  })
})
