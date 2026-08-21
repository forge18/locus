import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { NEXT_VERSION, SAVE_LABEL, useAgentDefs } from '../../src/data/agent-defs'
import { read } from '../css'

const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/save-is-a-version', () => {
  it('reads "Save as v5"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-save').textContent).toBe('Save as v5')
  })

  it('never reads plain "Save"', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-save').textContent).not.toBe('Save')
    expect(SAVE_LABEL).toMatch(/^Save as v\d+$/)
  })

  it('names the next version, one past the current one', () => {
    expect(NEXT_VERSION).toBe(useAgentDefs().find((d) => d.name === 'builder')!.version + 1)
  })

  it('builds the label from the version rather than writing it out', () => {
    expect(read('fixtures/agent-defs.ts')).toContain('`Save as v${NEXT_VERSION}`')
  })

  it('says why in the source: a definition is immutable once a run references it', () => {
    expect(read('fixtures/agent-defs.ts')).toContain('immutable once a run references it')
  })
})
