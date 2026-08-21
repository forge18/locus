import { describe, expect, it } from 'vitest'
import { createNavStore } from '../../src/nav'

describe('nav/history-stack', () => {
  it('starts on one entry with nowhere to go', () => {
    const nav = createNavStore()
    expect(nav.history()).toEqual(['locus://tapestry/inbox'])
    expect(nav.canBack()).toBe(false)
    expect(nav.canForward()).toBe(false)
  })

  it('is a stack of locators, not of views', () => {
    const nav = createNavStore()
    nav.go('board')
    nav.go('sessions', { sessionId: '8f21' })
    expect(nav.history()).toEqual([
      'locus://tapestry/inbox',
      'locus://tapestry/board',
      'locus://tapestry/session/8f21',
    ])
  })

  it('walks back and forward through it', () => {
    const nav = createNavStore()
    nav.go('board')
    nav.go('runs')
    expect(nav.view()).toBe('runs')

    nav.back()
    expect(nav.view()).toBe('board')
    expect(nav.canForward()).toBe(true)

    nav.back()
    expect(nav.view()).toBe('inbox')
    expect(nav.canBack()).toBe(false)

    nav.forward()
    expect(nav.view()).toBe('board')
  })

  it('restores the params, not just the view', () => {
    const nav = createNavStore()
    nav.go('runs', { sessionId: '8f21', runId: '3c04' })
    nav.go('wiki')
    nav.back()
    expect(nav.view()).toBe('runs')
    expect(nav.params()).toEqual({ project: 'tapestry', sessionId: '8f21', runId: '3c04' })
  })

  it('discards the forward entries once you go somewhere else', () => {
    const nav = createNavStore()
    nav.go('board')
    nav.go('runs')
    nav.back()
    nav.go('wiki')
    expect(nav.history()).toEqual([
      'locus://tapestry/inbox',
      'locus://tapestry/board',
      'locus://tapestry/wiki',
    ])
    expect(nav.canForward()).toBe(false)
  })

  it('does not stack a repeat of where you already are', () => {
    const nav = createNavStore()
    nav.go('board')
    nav.go('board')
    nav.go('board')
    expect(nav.history()).toEqual(['locus://tapestry/inbox', 'locus://tapestry/board'])
  })

  it('is per store, which is per window', () => {
    const a = createNavStore()
    const b = createNavStore()
    a.go('runs')
    expect(a.history().length).toBe(2)
    expect(b.history().length).toBe(1)
    expect(b.view()).toBe('inbox')
  })

  it('goes nowhere at either end', () => {
    const nav = createNavStore()
    nav.back()
    nav.back()
    expect(nav.view()).toBe('inbox')
    nav.forward()
    expect(nav.view()).toBe('inbox')
  })
})
