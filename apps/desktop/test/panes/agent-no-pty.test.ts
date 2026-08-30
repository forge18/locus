import { expect, it } from 'vitest'
import { agentPaneTransport } from '../../src/panes/AgentPane'
import type { Pane, PaneKind } from '../../src/panes/manager'

it('streams agent events without a PTY transport', () => {
  expect(agentPaneTransport).toBe('event-channel')
})

it('retires the shell pane kind', () => {
  const panes: Pane[] = [
    { id: 'one', kind: 'agent', focusedAt: 1 },
    { id: 'two', kind: 'editor', focusedAt: 2 },
  ]
  expect(panes.map((pane) => pane.kind)).toEqual(['agent', 'editor'])
  // Spec acceptance 6: no terminal on a run. Re-adding 'shell' to PaneKind
  // makes the assignment below typecheck, failing on the unused directive.
  // @ts-expect-error 'shell' is no longer a PaneKind
  const retired: PaneKind = 'shell'
  expect(panes.map((pane) => pane.kind)).not.toContain(retired)
})
