import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ARTIFACT_LOCATOR } from '../../src/data/artifacts'
import { V2InboxView } from '../../src/screens/v2-dashboard'
import { MemoryArtifactsFixture } from '../../src/screens/memory/MemoryFixtures'

describe('artifact one viewer', () => {
  it('uses one locator from review, memory, and inbox', () => {
    const inbox = render(() => <V2InboxView />)
    const memory = render(() => <MemoryArtifactsFixture />)
    expect(inbox.getByTestId('v2-inbox').textContent).toContain(ARTIFACT_LOCATOR)
    expect(memory.getByTestId('v2-memory-artifacts').textContent).toContain(ARTIFACT_LOCATOR)
  })
})
