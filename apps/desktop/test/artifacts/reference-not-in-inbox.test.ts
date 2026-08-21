import { describe, expect, it } from 'vitest'
import { REFERENCE_KINDS } from '../../src/data/artifacts'
import { useInboxItems } from '../../src/data/inbox'

const referenceKinds = REFERENCE_KINDS.map((k) => k.kind)

describe('artifacts/reference-not-in-inbox', () => {
  it('has no inbox item of a reference kind', () => {
    for (const item of useInboxItems()) {
      expect(referenceKinds, item.id).not.toContain(item.artifactKind as string)
    }
  })

  it('carries only review kinds, or none at all, on an inbox item', () => {
    for (const item of useInboxItems()) {
      if (item.artifactKind === null) continue
      expect(['diff', 'plan', 'report', 'log', 'image', 'video', 'handoff'], item.id).toContain(
        item.artifactKind,
      )
    }
  })

  it('keeps the two sets disjoint', () => {
    const inboxKinds = new Set<string>(
      useInboxItems()
        .map((i) => i.artifactKind)
        .filter((k) => k !== null)
        .map((k) => k as string),
    )
    for (const kind of referenceKinds) {
      expect(inboxKinds.has(kind), kind).toBe(false)
    }
  })

  it('is why the split exists: the inbox is not a place for an agent’s own scratch', () => {
    expect(REFERENCE_KINDS.map((k) => k.label)).toEqual(['finding', 'payload'])
    expect(useInboxItems().length).toBeLessThan(10)
  })
})
