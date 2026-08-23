import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { SESSION_LIST_FOOTER } from '../../src/data/sessions'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)

describe('agents/list-footer', () => {
  it('states the sort rule', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-foot').textContent).toContain(
      'Sorted by needs-attention, then activity.',
    )
  })

  it('says selecting one does not close the others', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-foot').textContent).toContain(
      'Selecting one does not close the others',
    )
  })

  it('says a session you stopped watching is not a session you ended', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-foot').textContent).toContain(
      'a session you stopped watching is not a session you ended',
    )
  })

  it('renders it verbatim from one constant', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-foot').textContent).toBe(SESSION_LIST_FOOTER)
  })

  it('sits at the foot under a top hairline', () => {
    const { getByTestId } = mount()
    const list = getByTestId('session-list')
    expect(list.children[2]).toBe(getByTestId('session-list-foot'))
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.session-list-foot')!.body,
    ).toContain('border-top: 1px solid var(--border-subtle)')
  })
})
