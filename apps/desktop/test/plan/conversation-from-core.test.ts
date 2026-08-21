import { describe, expect, it } from 'vitest'
import { read } from '../css'

describe('plan/conversation-from-core', () => {
  it('subscribes to the source-neutral core event channel for ACP conversation updates', () => {
    const source = read('data/plan.ts')
    expect(source).toContain("import { streamFromCore } from '../transcript/from-core'")
    expect(source).toContain('subscribePlanConversationFromCore')
    expect(source).toContain('streamFromCore')
  })

  it('renders ACP messages delivered over IPC while preserving the fixture fallback', () => {
    const source = read('screens/plan/PlanView.tsx')
    expect(source).toContain('subscribePlanConversationFromCore')
    expect(source).toContain('onMount')
    expect(source).toContain('setMessages')
  })
})
