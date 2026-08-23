import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Message } from '../../src/screens/plan/Message'
import { usePlanConversation } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const auditor = usePlanConversation().find((m) => m.speaker === 'auditor')!
const interviewer = usePlanConversation().find((m) => m.speaker === 'interviewer')!

describe('plan/auditor-finding', () => {
  it('tints the auditor bubble red', () => {
    const { getByTestId } = render(() => <Message message={auditor} />)
    expect(getByTestId(`msg-${auditor.id}`).className).toContain('msg-auditor')
    const body = rule('.msg-auditor .msg-bubble').body
    expect(body).toContain('border-color: var(--status-danger)')
    expect(body).toContain('color-mix(in srgb, var(--status-danger) 8%, var(--surface-raised))')
  })

  it('leaves an ordinary agent bubble on the hairline', () => {
    const { getByTestId } = render(() => <Message message={interviewer} />)
    expect(getByTestId(`msg-${interviewer.id}`).className).not.toContain('msg-auditor')
    expect(rule('.msg-bubble').body).toContain('border: 1px solid var(--border-subtle)')
  })

  it('labels the finding, in --bad', () => {
    const { getByTestId } = render(() => <Message message={auditor} />)
    expect(getByTestId(`msg-finding-${auditor.id}`).textContent).toBe('Finding — missed question')
    expect(rule('.msg-auditor .msg-finding').body).toContain('color: var(--status-danger)')
  })

  it('carries the auditor caption naming its fresh context and its standard', () => {
    const { getByTestId } = render(() => <Message message={auditor} />)
    const caption = getByTestId(`msg-caption-${auditor.id}`).textContent!
    expect(caption).toContain('fresh context')
    expect(caption).toContain('29148')
  })

  it('says what is wrong, not just that something is', () => {
    const { getByTestId } = render(() => <Message message={auditor} />)
    expect(getByTestId(`msg-bubble-${auditor.id}`).textContent).toContain('is not defined')
  })
})
