import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { MemoryLongTermFixture, MemoryShortTermFixture, MemoryWikiFixture } from '../../src/demo/MemoryFixtures'
import { MailView } from '../../src/screens/mail/MailView'
import { COMPACTED_CONTEXT, RESIDENT_LAYERS, WIKI_KIND_CHIPS } from '../../src/data/knowledge'
import { MAIL_STATUSES, MAIL_VERBS } from '../../src/fixtures/mail'
import { useArtifact } from '../../src/data/artifacts'

describe('M0.7 knowledge revision fixtures', () => {
  it('keeps the resident prefix fixed and compacted rows fetchable', () => {
    const { getByTestId } = render(() => <MemoryShortTermFixture />)
    expect(RESIDENT_LAYERS.map((layer) => layer.name)).toEqual([
      'base-context', 'rules in scope', 'skills loaded', 'the live plan', 'recalled facts', 'tool results', 'assistant turns',
    ])
    for (const item of COMPACTED_CONTEXT) {
      expect(getByTestId('desktop-memory-short-term').querySelector(`[data-artifact-id="${item.artifactId}"]`)).toBeTruthy()
      expect(useArtifact(item.artifactId)?.id).toBe(item.artifactId)
    }
  })

  it('shows revision 1 and the curated revision 2 without a score on contradictions', () => {
    const { getByTestId, getByText } = render(() => <MemoryLongTermFixture />)
    expect(getByTestId('memory-curation').textContent).toContain('revision 1')
    expect(getByTestId('memory-curation').textContent).toContain('revision 2')
    expect(getByText('— contradicted · no score')).toBeTruthy()
  })

  it('wires recalled-fact curation and contradiction actions', () => {
    const { getByTestId, getByText } = render(() => <MemoryLongTermFixture />)
    fireEvent.click(getByTestId('edit-recalled-fact'))
    expect(getByTestId('memory-recalled-fact-editor')).toBeTruthy()
    fireEvent.click(getByTestId('memory-save-revision'))
    expect(getByTestId('memory-action-status').textContent).toContain('Revision 2 staged')
    fireEvent.click(getByText('Adjudicate'))
    expect(getByTestId('memory-action-status').textContent).toContain('Contradiction adjudicated')
  })

  it('uses All plus exactly the five visible wiki kinds, never Overview', () => {
    const { getByTestId } = render(() => <MemoryWikiFixture />)
    expect(WIKI_KIND_CHIPS.map((chip) => chip.label)).toEqual(['All', 'Decisions', 'Concepts', 'Entities', 'Sources', 'Syntheses'])
    expect(getByTestId('desktop-memory-wiki').querySelector('[data-kind="overview"]')).toBeNull()
  })

  it("renders mail's statuses, verbs, waiting invariant, and state controls", () => {
    const { getByTestId, getByText } = render(() => <MailView />)
    expect(MAIL_STATUSES.every((status) => getByTestId('mail').querySelector(`[data-status="${status}"]`) || getByTestId('mail').querySelector(`.mail-status-${status}`))).toBe(true)
    expect(MAIL_VERBS.every((verb) => getByText(`mail ${verb}`))).toBe(true)
    expect(getByTestId('mail-wait-banner').textContent).toContain('State is waiting, not idle. The idle guardrail will not fire.')
    expect(getByTestId('mail-unblock')).toBeTruthy()
  })
})
