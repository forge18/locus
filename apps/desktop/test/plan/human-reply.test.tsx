import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Message } from '../../src/screens/plan/Message'
import { usePlanConversation } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const you = usePlanConversation().find((m) => m.speaker === 'you')!
const mount = () => render(() => <Message message={you} />)

describe('plan/human-reply', () => {
  it('right-aligns your reply by reversing the row', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-${you.id}`).className).toContain('msg-you')
    expect(rule('.msg-you').body).toContain('flex-direction: row-reverse')
    expect(rule('.msg-you .msg-col').body).toContain('align-items: flex-end')
  })

  it('grounds it on --sf3, a step above the agents', () => {
    expect(rule('.msg-you .msg-bubble').body).toContain('background: var(--surface-elevated)')
  })

  it('caps it at 560px, narrower than an agent bubble', () => {
    expect(rule('.msg-you .msg-bubble').body).toContain('max-width: 560px')
    expect(rule('.msg-bubble').body).toContain('max-width: 600px')
  })

  it('captions it "you"', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-caption-${you.id}`).textContent).toBe('you')
  })

  it('carries the body it was given', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-bubble-${you.id}`).textContent).toContain('Provenance')
  })
})
