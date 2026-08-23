import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxDetail } from '../../src/screens/inbox/InboxDetail'
import { useInboxItems } from '../../src/data/inbox'
import { read, rules } from '../css'

const [gate] = useInboxItems()
const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = () =>
  render(() => (
    <InboxDetail item={gate} onApprove={() => {}} onSendBack={() => {}} onOpenWork={() => {}} />
  ))

describe('inbox/footer', () => {
  it('sits on the deep ground under a top hairline', () => {
    const body = rule('.inbox-footer')!.body
    expect(body).toContain('background: var(--surface-chrome)')
    expect(body).toContain('border-top: 1px solid var(--border-subtle)')
  })

  it('leads with the primary approve', () => {
    const { getByTestId } = mount()
    const approve = getByTestId('inbox-approve')
    expect(approve.textContent).toBe('Approve & release the loop')
    expect(approve.className).toContain('btn-primary')
  })

  it('follows with the secondary send-back', () => {
    const { getByTestId } = mount()
    const back = getByTestId('inbox-send-back')
    expect(back.textContent).toBe('Send back with comment')
    expect(back.className).toContain('btn-secondary')
  })

  it('carries the note about where things resolve and where work opens', () => {
    const { getByTestId } = mount()
    expect(getByTestId('inbox-footer-note').textContent).toBe(
      'Resolves here · the work opens where the work lives',
    )
    expect(rule('.inbox-footer-note')!.body).toContain('margin-left: auto')
  })

  it('draws the primary as an accent line, not a fill', () => {
    const primary = rules(read('ui/ui.css')).find((r) => r.selector === '.btn-primary')!
    expect(primary.body).toContain('border-color: var(--action-attention)')
    expect(primary.body).not.toMatch(/background/)
  })
})
