import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryShortTermFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory compaction', () => {
  it('identifies compacted output artifact handles', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(getByTestId('desktop-memory-short-term').getAttribute('data-compaction')).toBe('artifact-handles')
  })
})
