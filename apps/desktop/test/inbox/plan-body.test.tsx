import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxDetail } from '../../src/screens/inbox/InboxDetail'
import { useInboxItems } from '../../src/data/inbox'
import { read, rules } from '../css'

const [gate, ask] = useInboxItems()
const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = (item = gate) =>
  render(() => (
    <InboxDetail item={item} onApprove={() => {}} onSendBack={() => {}} onOpenWork={() => {}} />
  ))

describe('inbox/plan-body', () => {
  it('labels a gate body PLAN, in accent uppercase', () => {
    const { getByTestId } = mount()
    expect(getByTestId('inbox-body-label').textContent).toBe('Plan')
    expect(rule('.inbox-body-label')!.body).toContain('color: var(--ac)')
    expect(rule('.inbox-body-label')!.body).toContain('text-transform: uppercase')
  })

  it('numbers the steps', () => {
    const { getByTestId } = mount()
    const steps = getByTestId('inbox-steps')
    expect(steps.tagName).toBe('OL')
    expect(steps.querySelectorAll('li').length).toBe(gate.body.length)
  })

  it('sets them at 15px on a 1.6 line', () => {
    const body = rule('.inbox-steps')!.body
    expect(body).toContain('font-size: var(--t-row)')
    expect(body).toContain('line-height: 1.6')
  })

  it('sets an inline path in mono', () => {
    const { getByTestId } = mount()
    const code = getByTestId('inbox-steps').querySelector('code')!
    expect(code.className).toContain('mono')
    expect(code.textContent).toBe('crates/tapestry-core/src/notify.rs')
  })

  it('carries the info callout on --sf', () => {
    const { getByTestId } = mount()
    const callout = getByTestId('inbox-callout')
    expect(callout.querySelector('use')!.getAttribute('href')).toBe('#ph-info')
    expect(callout.textContent).toContain('kept out')
    expect(rule('.inbox-callout')!.body).toContain('background: var(--sf)')
  })

  it('shows no callout on an item that has none', () => {
    const { queryByTestId } = mount(ask)
    expect(queryByTestId('inbox-callout')).toBe(null)
  })

  it('labels a non-gate body by what it is, rather than calling everything a plan', () => {
    const { getByTestId } = mount(ask)
    expect(getByTestId('inbox-body-label').textContent).toBe('Question')
  })
})
