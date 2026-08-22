import { expect, it } from 'vitest'
import { agentPaneTransport } from '../../src/panes/AgentPane'

it('streams agent events without a PTY transport', () => {
  expect(agentPaneTransport).toBe('event-channel')
})
