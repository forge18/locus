import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ARTIFACT_LOCATOR } from '../../src/data/artifacts'
import { DesktopInboxView } from '../../src/screens/desktop-dashboard'
import { MemoryArtifactsFixture } from '../../src/screens/memory/MemoryFixtures'

describe('artifact one viewer', () => {
  it('uses one locator from review, memory, and inbox', () => {
    const inbox = render(() => <DesktopInboxView />)
    const memory = render(() => <MemoryArtifactsFixture />)
    expect(inbox.getByTestId('desktop-inbox').textContent).toContain(ARTIFACT_LOCATOR)
    expect(memory.getByTestId('desktop-memory-artifacts').textContent).toContain(ARTIFACT_LOCATOR)
  })
})
