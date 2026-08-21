import { describe, expect, it } from 'vitest'
import { createNavStore } from '../../src/nav'

describe('nav/store-derives', () => {
  it('derives the category from the view', () => {
    const nav = createNavStore()
    expect(nav.category()).toBe('dashboard')
    nav.go('runs')
    expect(nav.category()).toBe('review')
  })

  it('derives the category label the rail and tab bar show', () => {
    const nav = createNavStore({ view: 'status' })
    expect(nav.categoryLabel()).toBe('Inbox')
    nav.go('canvas')
    expect(nav.categoryLabel()).toBe('Workshop')
  })

  it('derives the locator, and the path form the bars render', () => {
    const nav = createNavStore({ project: 'weaver', view: 'telemetry' })
    expect(nav.locator()).toBe('locus://weaver/telemetry')
    expect(nav.locatorPath()).toBe('weaver/telemetry')
  })

  it('derives the visible tab set', () => {
    const nav = createNavStore()
    expect(nav.tabs().map((t) => t.label)).toEqual(['Inbox', 'Status'])
    nav.go('extensions')
    expect(nav.tabs().map((t) => t.label)).toEqual(['Extensions', 'Workflow', 'Harnesses'])
    nav.go('plan')
    expect(nav.tabs()).toEqual([])
  })

  it('normalizes params through the grammar, so nothing survives that a locator cannot carry', () => {
    const nav = createNavStore()
    nav.go('sessions', { sessionId: '8f21' })
    expect(nav.params()).toEqual({ project: 'tapestry', sessionId: '8f21' })
    // Moving on drops the id rather than dragging it to a view where it means nothing.
    nav.go('board')
    expect(nav.params()).toEqual({ project: 'tapestry' })
  })

  it('keeps the project across a view change — it is a scope, not a destination', () => {
    const nav = createNavStore({ project: 'weaver' })
    nav.go('runs')
    nav.go('wiki')
    expect(nav.params().project).toBe('weaver')
  })
})
