import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { useSessionDetails } from '../../src/data/sessions'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)

describe('agents/list-header', () => {
  it('is headed AGENTS', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-head').textContent).toContain('Agents')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.session-list-head')!.body,
    ).toContain('text-transform: uppercase')
  })

  it('counts what is running, from the data', () => {
    const { getByTestId } = mount()
    const running = useSessionDetails().filter((s) => s.status !== 'done').length
    expect(getByTestId('session-list-count').textContent).toBe(
      `${running} running · one session each`,
    )
  })

  it('says one session each — the mapping the whole design depends on', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list-count').textContent).toContain('one session each')
  })

  it('offers a funnel and an accent sort', () => {
    const { getByLabelText } = mount()
    expect(getByLabelText('Filter').querySelector('use')!.getAttribute('href')).toBe('#ph-funnel')
    const sort = getByLabelText('Sort')
    expect(sort.querySelector('use')!.getAttribute('href')).toBe('#ph-sort-ascending')
    expect(sort.getAttribute('style')).toContain('var(--ac)')
  })

  it('sits above the list under a bottom hairline', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list').children[0]).toBe(getByTestId('session-list-head'))
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.session-list-head')!.body,
    ).toContain('border-bottom: 1px solid var(--line)')
  })
})
