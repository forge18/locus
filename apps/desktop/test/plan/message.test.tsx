import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Message } from '../../src/screens/plan/Message'
import { usePlanConversation } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const [interviewer, , researcher] = usePlanConversation()
const mount = (m = interviewer) => render(() => <Message message={m} />)

describe('plan/message', () => {
  it('shows a 22px rounded avatar carrying mono initials', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-avatar-${interviewer.id}`).textContent).toBe('IN')
    const body = rule('.msg-avatar').body
    expect(body).toContain('width: 22px')
    expect(body).toContain('height: 22px')
    expect(body).toContain('border-radius: var(--r-sm)')
    expect(body).toContain('font-family: var(--fm)')
  })

  it('captions the message with its role', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-caption-${interviewer.id}`).textContent).toBe('interviewer · plan')
  })

  it('bubbles the body on --sf, capped at 600px', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-bubble-${interviewer.id}`).textContent).toBe(interviewer.body)
    expect(rule('.msg-bubble').body).toContain('max-width: 600px')
    expect(rule('.msg-bubble').body).toContain('background: var(--surface-raised)')
  })

  it('shows the fact row when the speaker went and looked', () => {
    const { getByTestId } = mount(researcher)
    const facts = getByTestId(`msg-facts-${researcher.id}`)
    expect(facts.textContent).toContain('3 repos indexed')
    expect(facts.textContent).toContain('4 wiki pages')
    expect(facts.textContent).toContain('2 decisions')
  })

  it('shows no fact row when there is nothing to show', () => {
    const { queryByTestId } = mount(interviewer)
    expect(queryByTestId(`msg-facts-${interviewer.id}`)).toBe(null)
  })

  it('reports the speaker to the DOM', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`msg-${interviewer.id}`).getAttribute('data-speaker')).toBe('interviewer')
  })
})
