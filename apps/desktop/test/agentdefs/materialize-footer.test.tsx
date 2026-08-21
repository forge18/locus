import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentDefsView } from '../../src/screens/workshop/AgentDefsView'
import { MATERIALIZE_TARGET, useAgentMaterialization } from '../../src/data/agent-defs'
import { useExtensionCounts, useHarnessSummary } from '../../src/data/harnesses'
import { read } from '../css'

const mount = () => render(() => <AgentDefsView onNavigate={() => {}} />)

describe('agentdefs/materialize-footer', () => {
  it('names the materialization target, in mono', () => {
    const { getByTestId } = mount()
    const foot = getByTestId('agentdefs-foot')
    expect(foot.textContent).toContain(MATERIALIZE_TARGET)
    expect(foot.querySelector('.mono')!.textContent).toBe('/locus/config/agents/')
  })

  it('names how many harnesses it reaches, from the registry', () => {
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-foot').textContent).toContain(
      `${useHarnessSummary().harnesses} harnesses`,
    )
  })

  it('names how many take it weaker than native, for this type only', () => {
    const agents = useExtensionCounts().find((c) => c.type === 'agents')!
    expect(useAgentMaterialization().downgraded).toBe(agents.downgraded)
    const { getByTestId } = mount()
    expect(getByTestId('agentdefs-foot').textContent).toContain(`${agents.downgraded} downgraded`)
  })

  it('counts the agents type, not the whole registry', () => {
    const agents = useExtensionCounts().find((c) => c.type === 'agents')!
    expect(agents.downgraded).toBeLessThan(useHarnessSummary().downgrades)
  })

  it('computes both rather than writing them down', () => {
    expect(read('data/agent-defs.ts')).toContain('EXTENSION_COUNTS.find')
    expect(read('data/agent-defs.ts')).toContain('HARNESS_COUNT')
  })
})
