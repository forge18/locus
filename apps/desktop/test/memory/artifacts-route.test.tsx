import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryArtifactsFixture } from '../../src/screens/memory/MemoryFixtures'

describe('Memory artifacts route', () => {
  it('identifies grouped artifact fixture route', () => {
    const { getByTestId } = render(() => <MemoryArtifactsFixture />)
    expect(getByTestId('v2-memory-artifacts').getAttribute('data-artifact-groups')).toBe('review-reference')
  })
})
