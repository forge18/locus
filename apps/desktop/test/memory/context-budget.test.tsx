import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryShortTermFixture } from '../../src/demo/MemoryFixtures'

describe('Memory context budget', () => {
  it('identifies the context ceiling and compaction threshold', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(getByTestId('desktop-memory-short-term').getAttribute('data-context-budget')).toBe('120k')
  })
})
