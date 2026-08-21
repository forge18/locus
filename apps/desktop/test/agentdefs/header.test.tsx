import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { DIFF_LABEL, PROVENANCE, SAVE_LABEL } from '../../src/data/agent-defs'
import { read, rules } from '../css'

const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/header', () => {
  it('shows the filename in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-file').textContent).toBe('builder.md')
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.agentdefs-file')!.body,
    ).toContain('font-family: var(--fm)')
  })

  it('shows the provenance in mono: version, edit age, and who is using it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-provenance').textContent).toBe(PROVENANCE)
    expect(PROVENANCE).toContain('v4')
    expect(PROVENANCE).toContain('edited 2h ago')
    expect(PROVENANCE).toContain('used by 5 sessions')
  })

  it('offers Diff v3 as the secondary', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-diff').textContent).toBe(DIFF_LABEL)
    expect(getByTestId('agentdefs-diff').className).toContain('btn-secondary')
  })

  it('offers the save action as the primary', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-save').textContent).toBe(SAVE_LABEL)
    expect(getByTestId('agentdefs-save').className).toContain('btn-primary')
  })

  it('sits under a bottom hairline', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.agentdefs-head')!.body,
    ).toContain('border-bottom: 1px solid var(--line)')
  })
})
