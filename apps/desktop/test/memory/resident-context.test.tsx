import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryShortTermFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory resident context', () => {
  it('labels resident prompt layers and cache state', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(getByTestId('v2-memory-short-term').getAttribute('data-context-layers')).toBe('resident')
  })
})
