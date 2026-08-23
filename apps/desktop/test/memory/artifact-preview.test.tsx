import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryArtifactsFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory artifact preview', () => {
  it('identifies preview comments and review state', () => {
    const { getByTestId } = render(() => <MemoryArtifactsFixture />)
    expect(getByTestId('v2-memory-artifacts').getAttribute('data-artifact-preview')).toBe('comments-review')
  })
})
