import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { useAgentDefs, useDefaultAgentDef } from '../../src/data/agent-defs'
import { read, rules } from '../css'

const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/list', () => {
  it('lists the six definitions', () => {
    const { getByTestId } = mount()
    expect(useAgentDefs().map((d) => d.name)).toEqual([
      'builder',
      'reviewer',
      'interviewer',
      'researcher',
      'auditor',
      'keeper',
    ])
    for (const def of useAgentDefs()) {
      expect(getByTestId(`agentdef-${def.name}`), def.name).toBeTruthy()
    }
  })

  it('shows each version in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdef-version-builder').textContent).toBe('v4')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.agentdefs-version')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('marks exactly one selected, and rings it with --line2 over --sf2', () => {
    const { getByTestId } = mount()
    const marked = getByTestId('agentdefs-side').querySelectorAll('[aria-selected="true"]')
    expect(marked.length).toBe(1)
    expect(marked[0].getAttribute('data-testid')).toBe(`agentdef-${useDefaultAgentDef()}`)
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === ".agentdefs-row[aria-selected='true']",
    )!.body
    expect(body).toContain('background: var(--sf2)')
    expect(body).toContain('inset 0 0 0 1px var(--line2)')
  })

  it('opens on builder', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-file').textContent).toBe('builder.md')
  })

  it('swaps the file when another is picked', () => {
    const { getByTestId } = mount()
    getByTestId('agentdef-auditor').click()
    expect(getByTestId('agentdefs-file').textContent).toBe('auditor.md')
    expect(getByTestId('agentdef-auditor').getAttribute('aria-selected')).toBe('true')
  })
})
