import { expect, it } from 'vitest'
import { AgentPane } from '../../src/panes/AgentPane'
import { ShellPane } from '../../src/panes/ShellPane'

it('uses separate shell and agent pane components', () => {
  expect(ShellPane).not.toBe(AgentPane)
})
