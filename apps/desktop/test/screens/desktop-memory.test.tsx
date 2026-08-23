import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import {
  MemoryArtifactsFixture,
  MemoryLongTermFixture,
  MemoryShortTermFixture,
  MemoryWikiFixture,
} from '../../src/screens/memory/MemoryFixtures'
import { read, rules } from '../css'

describe('screens/desktop-memory', () => {
  it('renders the short-term context window with resident prefix and compaction sections', () => {
    const { getByTestId, getByText } = render(() => <MemoryShortTermFixture />)

    expect(getByTestId('desktop-memory-short-term')).toBeTruthy()
    expect(getByText(/^The context window\. Nothing here is stored/)).toBeTruthy()
    expect(getByText('Resident now')).toBeTruthy()
    expect(getByText('Compacted out')).toBeTruthy()
    expect(getByText('Prefix cache')).toBeTruthy()
    expect(getByText('What survives the iteration')).toBeTruthy()
  })

  it('renders durable facts with scope, provenance, decay, and a resolvable contradiction', () => {
    const { getByTestId, getByText } = render(() => <MemoryLongTermFixture />)

    expect(getByTestId('desktop-memory-long-term')).toBeTruthy()
    expect(getByText('never cross-project')).toBeTruthy()
    expect(getByText('Why this is trusted')).toBeTruthy()
    expect(getByText('Confidence over time')).toBeTruthy()
    expect(getByText('Adjudicate')).toBeTruthy()
    expect(getByText('locus memory explain')).toBeTruthy()
  })

  it('renders the artifact viewer with review and reference groups plus comment steering', () => {
    const { getByTestId, getByText } = render(() => <MemoryArtifactsFixture />)

    expect(getByTestId('desktop-memory-artifacts')).toBeTruthy()
    expect(getByText('Review artifacts')).toBeTruthy()
    expect(getByText('Reference · never in the inbox')).toBeTruthy()
    expect(getByText('one viewer per kind · three entry points')).toBeTruthy()
    expect(getByText('Comments steer the agent')).toBeTruthy()
  })

  it('renders the curated wiki with provenance, graph, contradictions, and lint findings', () => {
    const { getByTestId, getByText } = render(() => <MemoryWikiFixture />)

    expect(getByTestId('desktop-memory-wiki')).toBeTruthy()
    expect(getByText('Ingest a document')).toBeTruthy()
    expect(getByText('Provenance')).toBeTruthy()
    expect(getByText('Graph')).toBeTruthy()
    expect(getByText('Contradictions')).toBeTruthy()
    expect(getByText('locus wiki lint')).toBeTruthy()
  })

  it('uses the semantic data ramp for every context and confidence bar', () => {
    const memoryRule = (selector: string) => rules(read('screens/memory/memory.css')).find((entry) => entry.selector === selector)!

    expect(memoryRule('.desktop-memory-bar').body).toContain('background: var(--data-1)')
    expect(memoryRule('.desktop-memory-bar > span').body).toContain('background: var(--data-3)')
    expect(memoryRule('.desktop-memory-confidence > span').body).toContain('background: var(--data-2)')
    expect(memoryRule('.desktop-memory-bar > span').body).not.toContain('var(--action-attention)')
    expect(memoryRule('.desktop-memory-confidence > span').body).not.toContain('var(--action-attention)')
  })
})
