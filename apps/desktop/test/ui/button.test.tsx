import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Button } from '../../src/ui/Button'

const btn = (el: HTMLElement) => el.querySelector('button')!

describe('ui/button', () => {
  it('defaults to secondary', () => {
    const { container } = render(() => <Button>Send back with comment</Button>)
    expect(btn(container).className).toContain('btn-secondary')
  })

  it('carries each variant as its own class', () => {
    for (const v of ['primary', 'secondary', 'ghost'] as const) {
      const { container } = render(() => <Button variant={v}>x</Button>)
      expect(btn(container).className).toContain(`btn-${v}`)
    }
  })

  it('fills its container when block is set', () => {
    const { container } = render(() => <Button block>Approve — 4 tasks to the board</Button>)
    expect(btn(container).className).toContain('btn-block')
  })

  it('keeps a caller class alongside its own', () => {
    const { container } = render(() => <Button class="mono">locus://</Button>)
    const c = btn(container).className
    expect(c).toContain('btn')
    expect(c).toContain('mono')
  })

  it('passes clicks and disabled through to the element', () => {
    let clicks = 0
    const { getByText } = render(() => <Button onClick={() => clicks++}>Commit</Button>)
    getByText('Commit').click()
    expect(clicks).toBe(1)

    const off = render(() => <Button disabled>Push</Button>)
    expect(btn(off.container).disabled).toBe(true)
  })
})
