import { describe, expect, it } from 'vitest'
import { useSessionDetails } from '../../src/data/sessions'
import { SESSION_DETAILS } from '../../src/fixtures/sessions'

describe('agents/sort-order', () => {
  it('puts the stuck session first even though it is the least recently active', () => {
    const sorted = useSessionDetails()
    expect(sorted[0].status).toBe('stuck')

    const byActivity = [...SESSION_DETAILS].sort((a, b) => a.idleMinutes - b.idleMinutes)
    expect(byActivity[0].status).not.toBe('stuck')
    expect(byActivity.map((s) => s.id)).not.toEqual(sorted.map((s) => s.id))
  })

  it('orders stuck, then waiting, then idle, then running', () => {
    const rank: Record<string, number> = { stuck: 0, waiting: 1, idle: 2, running: 3 }
    const seen = useSessionDetails().map((s) => rank[s.status])
    expect(seen).toEqual([...seen].sort((a, b) => a - b))
  })

  it('breaks ties by activity, most recent first', () => {
    const running = useSessionDetails().filter((s) => s.status === 'running')
    expect(running.map((s) => s.idleMinutes)).toEqual(
      [...running.map((s) => s.idleMinutes)].sort((a, b) => a - b),
    )
  })

  it('never sorts by project or by name', () => {
    const projects = useSessionDetails().map((s) => s.project)
    expect(projects).not.toEqual([...projects].sort())
  })

  it('is the same rule the strip uses — it is the same question', () => {
    const sorted = useSessionDetails()
    expect(sorted[0].status).toBe('stuck')
    expect(sorted[sorted.length - 1].status).toBe('running')
  })
})
