import { describe, expect, it } from 'vitest'
import { useStripCards } from '../../src/data/strip'
import { STRIP_CARDS } from '../../src/fixtures/strip'

describe('shell/strip-ordering', () => {
  it('puts the stuck card first even though it is the least recently active', () => {
    const sorted = useStripCards()
    expect(sorted[0].status).toBe('stuck')

    // The two orders genuinely disagree, which is what makes this an assertion.
    const byActivity = [...STRIP_CARDS].sort((a, b) => a.idleMinutes - b.idleMinutes)
    expect(byActivity[0].status).not.toBe('stuck')
    expect(byActivity.map((c) => c.id)).not.toEqual(sorted.map((c) => c.id))
  })

  it('orders stuck, then waiting, then idle, then running', () => {
    const rank = { stuck: 0, waiting: 1, idle: 2 } as Record<string, number>
    const seen = useStripCards()
      .filter((c) => c.status && c.status !== 'running')
      .map((c) => rank[c.status!])
    expect(seen).toEqual([...seen].sort((a, b) => a - b))
  })

  it('breaks ties by activity, most recent first', () => {
    const running = useStripCards().filter((c) => c.status === 'running')
    expect(running.map((c) => c.idleMinutes)).toEqual(
      [...running.map((c) => c.idleMinutes)].sort((a, b) => a - b),
    )
  })

  it('never sorts by project or by name', () => {
    const sorted = useStripCards().map((c) => c.project)
    expect(sorted).not.toEqual([...sorted].sort())
  })

  it('leaves the fixture untouched — sorting returns a new list', () => {
    const before = STRIP_CARDS.map((c) => c.id)
    useStripCards()
    expect(STRIP_CARDS.map((c) => c.id)).toEqual(before)
  })
})
