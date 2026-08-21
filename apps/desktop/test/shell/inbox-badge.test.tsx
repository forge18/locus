import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Rail } from '../../src/shell/Rail'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('shell/shell.css')).find((r) => r.selector === sel)

describe('shell/inbox-badge', () => {
  it('shows the count on the Inbox item', () => {
    const { getByTestId } = render(() => <Rail view="plan" onNavigate={() => {}} inboxCount={3} />)
    expect(getByTestId('inbox-badge').textContent).toBe('3')
    expect(getByTestId('rail-dashboard').contains(getByTestId('inbox-badge'))).toBe(true)
  })

  it('disappears at zero — silence is legible from anywhere', () => {
    const { queryByTestId } = render(() => (
      <Rail view="plan" onNavigate={() => {}} inboxCount={0} />
    ))
    expect(queryByTestId('inbox-badge')).toBe(null)
  })

  it('is a 15px accent pill sitting top 5 / right 9', () => {
    const body = rule('.rail-badge')!.body
    expect(body).toContain('position: absolute')
    expect(body).toContain('top: 5px')
    expect(body).toContain('right: 9px')
    expect(body).toContain('min-width: 15px')
    expect(body).toContain('height: 15px')
    expect(body).toContain('background: var(--ac)')
  })

  it('inks the numeral in the app ground and carries the one 700 weight', () => {
    const body = rule('.rail-badge')!.body
    expect(body).toContain('color: var(--ac-ink)')
    expect(read('styles/tokens.css')).toContain('--ac-ink: var(--bg)')
    expect(body).toContain('font-weight: 700')
  })
})
