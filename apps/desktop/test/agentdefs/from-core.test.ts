import { describe, expect, it } from 'vitest'
import { read } from '../css'

describe('agentdefs/from-core', () => {
  it('loads definition summaries and selected content through the Tauri IPC boundary', () => {
    const source = read('data/agent-defs.ts')
    expect(source).toContain("invoke<AgentDefSummary[]>('agent_defs_list')")
    expect(source).toContain("invoke<CoreAgentDefinition>('agent_def', { name })")
  })

  it('refreshes the Workshop screen from core-owned definitions after mount', () => {
    const source = read('screens/workshop/AgentDefsView.tsx')
    expect(source).toContain('fetchAgentDefsFromCore')
    expect(source).toContain('fetchAgentDefFromCore')
  })
})
