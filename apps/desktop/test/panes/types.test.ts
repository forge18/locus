import { expect, it } from 'vitest'
import { agentPaneTransport } from '../../src/panes/AgentPane'
import { reachesTerminal, terminalOptions } from '../../src/panes/shell-config'

it('panes/shell-pty', () => expect(terminalOptions.macOptionIsMeta).toBe(true))
it('panes/agent-no-pty', () => expect(agentPaneTransport).toBe('event-channel'))
it('panes/types-are-distinct', () => expect(agentPaneTransport).not.toBe('pty-channel'))
it('panes/vim-survives', () => expect(reachesTerminal(new KeyboardEvent('keydown', { key: 'v', altKey: true }))).toBe(true))
it('panes/cmd-chords', () => expect(reachesTerminal(new KeyboardEvent('keydown', { key: 'k', metaKey: true }))).toBe(false))
it('panes/ime', () => expect(reachesTerminal(new KeyboardEvent('compositionstart', { key: 'Dead' }))).toBe(true))
