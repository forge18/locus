import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { WantsAttention } from '../../src/screens/status/WantsAttention'
import { useWantsAttention } from '../../src/data/status'

const mount = () => render(() => <WantsAttention rows={useWantsAttention()} />)

describe('status/waiting-not-idle', () => {
  it('says "waiting: gate — not idle" in as many words', () => {
    const { getByTestId } = mount()
    expect(getByTestId('attention-waiting-detail').textContent).toBe('waiting: gate — not idle')
  })

  it('says something different on the idle row', () => {
    const { getByTestId } = mount()
    expect(getByTestId('attention-idle-detail').textContent).toBe(
      'idle 3m · no event on the stream',
    )
  })

  it('gives them different icons, so they are distinct without reading', () => {
    const { getByTestId } = mount()
    const waiting = getByTestId('attention-waiting').querySelector('use')!.getAttribute('href')
    const idle = getByTestId('attention-idle').querySelector('use')!.getAttribute('href')
    expect(waiting).not.toBe(idle)
  })

  it('marks the kind in the DOM, so nothing downstream has to guess from the copy', () => {
    const { getByTestId } = mount()
    expect(getByTestId('attention-waiting').getAttribute('data-kind')).toBe('waiting')
    expect(getByTestId('attention-idle').getAttribute('data-kind')).toBe('idle')
  })

  it('keeps the distinction in the fixture, not only in the rendering', () => {
    const rows = useWantsAttention()
    expect(rows.find((r) => r.kind === 'waiting')!.detail).toContain('not idle')
    expect(rows.find((r) => r.kind === 'idle')!.detail).not.toContain('gate')
  })
})
