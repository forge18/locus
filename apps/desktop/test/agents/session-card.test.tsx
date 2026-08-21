import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { SessionCard } from '../../src/screens/automate/SessionCard'
import { useSessionDetail } from '../../src/data/sessions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const session = useSessionDetail('sd-tapestry')!
const unknown = useSessionDetail('sd-texere')!
const mount = (s = session) =>
  render(() => <SessionCard session={s} selected={false} onSelect={() => {}} />)

describe('agents/session-card', () => {
  it('leads with a status dot', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`session-dot-${session.id}`).className).toContain('session-dot-running')
  })

  it('shows the project at 19px/500 and the agent in mono', () => {
    const { getByTestId } = mount()
    const card = getByTestId(`session-card-${session.id}`)
    expect(card.querySelector('.session-project')!.textContent).toBe('tapestry')
    expect(card.querySelector('.session-agent')!.textContent).toBe('builder@4')
    expect(rule('.session-project').body).toContain('font-size: var(--t-row)')
    expect(rule('.session-agent').body).toContain('font-family: var(--fm)')
  })

  it('shows the role', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`session-card-${session.id}`).querySelector('.session-role')!.textContent).toBe(
      'builder',
    )
  })

  it('right-aligns the tokens in mono, and says unknown where none were reported', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`session-tokens-${session.id}`).textContent).toBe('41.2k')
    expect(rule('.session-tokens').body).toContain('margin-left: auto')

    const other = render(() => <SessionCard session={unknown} selected={false} onSelect={() => {}} />)
    expect(other.getByTestId(`session-tokens-${unknown.id}`).textContent).toBe('unknown')
  })

  it('shows the task at 14px and 76% opacity', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`session-task-${session.id}`).textContent).toBe(session.task)
    const body = rule('.session-task').body
    expect(body).toContain('font-size: var(--t-body)')
    expect(body).toMatch(/opacity:\s*\.76/)
  })

  it('shows a status chip, the current tool in mono, and the run count on the right', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`session-chip-${session.id}`).textContent).toBe('running')
    expect(getByTestId(`session-tool-${session.id}`).textContent).toBe('edit_file')
    expect(getByTestId(`session-runs-${session.id}`).textContent).toBe('1 run')
    expect(rule('.session-tool').body).toContain('font-family: var(--fm)')
    expect(rule('.session-runs').body).toContain('margin-left: auto')
  })

  it('says "no tool" between tools rather than leaving a gap', () => {
    const { getByTestId } = render(() => (
      <SessionCard session={unknown} selected={false} onSelect={() => {}} />
    ))
    expect(getByTestId(`session-tool-${unknown.id}`).textContent).toBe('no tool')
  })
})
