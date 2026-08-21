import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxDetail } from '../../src/screens/inbox/InboxDetail'
import { useInboxItems } from '../../src/data/inbox'
import { read, rules } from '../css'

const [gate] = useInboxItems()
const mount = (onApprove: (c: string) => void = () => {}) =>
  render(() => (
    <InboxDetail item={gate} onApprove={onApprove} onSendBack={() => {}} onOpenWork={() => {}} />
  ))

describe('inbox/comment-box', () => {
  it('captions the box with what a comment does', () => {
    const { getByTestId } = mount()
    expect(getByTestId('inbox-comment-caption').textContent).toBe(
      'Comment steers the agent that made it',
    )
  })

  it('is a textarea on the --sf input ground', () => {
    const { getByTestId } = mount()
    const box = getByTestId('inbox-comment')
    expect(box.tagName).toBe('TEXTAREA')
    expect(box.className).toContain('input')
    const rule = rules(read('ui/ui.css')).find((r) => r.selector === '.input')!
    expect(rule.body).toContain('background: var(--sf)')
  })

  it('starts at 64px and grows', () => {
    const rule = rules(read('ui/ui.css')).find((r) => r.selector === 'textarea.input')!
    expect(rule.body).toContain('min-height: 64px')
    expect(rule.body).toContain('resize: vertical')
  })

  it('is optional — approving with nothing typed still works', () => {
    let seen: string | null = null
    const { getByTestId } = mount((c) => (seen = c))
    getByTestId('inbox-approve').click()
    expect(seen).toBe('')
  })

  it('hands what was typed to the approve action', () => {
    let seen: string | null = null
    const { getByTestId } = mount((c) => (seen = c))
    const box = getByTestId('inbox-comment') as HTMLTextAreaElement
    box.value = 'Keep the HTTP sink out.'
    box.dispatchEvent(new Event('input', { bubbles: true }))
    getByTestId('inbox-approve').click()
    expect(seen).toBe('Keep the HTTP sink out.')
  })
})
