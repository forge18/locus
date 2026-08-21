import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { parse } from '../../src/nav'
import { useSessionDetails } from '../../src/data/sessions'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)
const first = useSessionDetails()[0]

describe('agents/transcript-header', () => {
  it('carries the dot, project, agent, role and task', () => {
    const { getByTestId } = mount()
    const head = getByTestId('transcript-head')
    expect(head.querySelector('.session-dot')).not.toBe(null)
    expect(head.querySelector('.session-project')!.textContent).toBe(first.project)
    expect(head.querySelector('.session-agent')!.textContent).toBe(first.agent)
    expect(head.querySelector('.session-role')!.textContent).toBe(first.role)
    expect(head.textContent).toContain(first.task)
  })

  it('shows the status chip', () => {
    const { getByTestId } = mount()
    expect(getByTestId('transcript-status').textContent).toBe(first.status)
  })

  it('shows a mono locator that parses', () => {
    const { getByTestId } = mount()
    const locator = getByTestId('transcript-locator')
    expect(() => parse(locator.textContent!)).not.toThrow()
    expect(parse(locator.textContent!).kind).toBe('session')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.transcript-locator')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('offers detach and minimize, in that order', () => {
    const { getByTestId } = mount()
    expect(getByTestId('transcript-detach').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-arrows-out-simple',
    )
    expect(getByTestId('transcript-minimize').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-minus',
    )
  })

  it('names both controls for a reader who cannot see the glyphs', () => {
    const { getByLabelText } = mount()
    expect(getByLabelText('Detach')).toBeTruthy()
    expect(getByLabelText('Minimize')).toBeTruthy()
  })
})
