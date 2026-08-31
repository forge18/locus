import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryShortTermFixture } from '../../src/demo/MemoryFixtures'

describe('Memory resident context', () => {
  it('labels resident prompt layers and cache state', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(getByTestId('desktop-memory-short-term').getAttribute('data-context-layers')).toBe('resident')
  })
})
