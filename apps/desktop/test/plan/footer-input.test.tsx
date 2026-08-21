import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { PlanView } from '../../src/screens/plan/PlanView'
import { ACP_LABEL } from '../../src/data/plan'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <PlanView />)

describe('plan/footer-input', () => {
  it('offers a place to answer the interviewer', () => {
    const { getByTestId } = mount()
    expect(getByTestId('plan-input').textContent).toContain('Answer the interviewer')
  })

  it('blinks an accent caret in it', () => {
    const { getByTestId } = mount()
    const caret = getByTestId('plan-caret')
    expect(caret.className).toContain('blink')
    expect(rule('.plan-caret').body).toContain('color: var(--ac)')
    expect(read('styles/motion.css')).toContain('.blink { animation: blink 1.1s')
  })

  it('names the transport in mono — this conversation is an ACP session', () => {
    const { getByTestId } = mount()
    const acp = getByTestId('plan-acp')
    expect(acp.textContent).toBe(ACP_LABEL)
    expect(acp.textContent).toBe('ACP · session/prompt')
    expect(rule('.plan-acp').body).toContain('font-family: var(--fm)')
  })

  it('sits under a top hairline, outside the scrolling messages', () => {
    const { getByTestId } = mount()
    expect(rule('.plan-convo-footer').body).toContain('border-top: 1px solid var(--line)')
    expect(getByTestId('plan-messages').contains(getByTestId('plan-footer'))).toBe(false)
  })
})
