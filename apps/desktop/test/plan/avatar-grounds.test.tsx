import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Message } from '../../src/screens/plan/Message'
import { usePlanConversation } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const messages = usePlanConversation()
const agent = messages.find((m) => m.speaker === 'interviewer')!
const auditor = messages.find((m) => m.speaker === 'auditor')!

describe('plan/avatar-grounds', () => {
  it('grounds an agent avatar on --blue', () => {
    const { getByTestId } = render(() => <Message message={agent} />)
    expect(getByTestId(`msg-avatar-${agent.id}`).className).not.toContain('msg-avatar-auditor')
    expect(rule('.msg-avatar').body).toContain('background: var(--blue)')
  })

  it('grounds the auditor on the deep amber, which is its own colour', () => {
    const { getByTestId } = render(() => <Message message={auditor} />)
    expect(getByTestId(`msg-avatar-${auditor.id}`).className).toContain('msg-avatar-auditor')
    expect(rule('.msg-avatar-auditor').body).toContain('background: var(--ac-deep)')
  })

  it('derives that amber from --ac, so retheming carries it', () => {
    expect(read('styles/tokens.css')).toContain('--ac-deep: color-mix(in srgb, var(--ac) 36%, #000000)')
  })

  it('tells the three speakers apart without reading them', () => {
    const rendered = messages.map((m) => {
      const { getByTestId, unmount } = render(() => <Message message={m} />)
      const el = getByTestId(`msg-${m.id}`)
      const signature = [
        el.className,
        el.querySelector('.msg-avatar')?.className ?? 'no-avatar',
      ].join('|')
      unmount()
      return { speaker: m.speaker, signature }
    })
    const bySpeaker = new Map(rendered.map((r) => [r.speaker, r.signature]))
    expect(new Set(bySpeaker.values()).size).toBe(bySpeaker.size)
  })

  it('gives you no avatar at all — you are the one reading', () => {
    const you = messages.find((m) => m.speaker === 'you')!
    const { queryByTestId } = render(() => <Message message={you} />)
    expect(queryByTestId(`msg-avatar-${you.id}`)).toBe(null)
  })
})
