import { invoke } from '@tauri-apps/api/core'
import { expect, it, vi } from 'vitest'
import {
  detachPane,
  detachedMode,
  detachedPaneRunId,
} from '../../src/panes/detach'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

it('opens the same app in detached mode in a new window', () => {
  history.replaceState({}, '', '/?detached=true&run=run-1')
  expect(detachedMode()).toBe(true)
  expect(detachedPaneRunId()).toBe('run-1')
})

it('passes the run identity to the host window command', async () => {
  vi.mocked(invoke).mockResolvedValue(undefined)
  await detachPane('pane-1', 'run-1')
  expect(invoke).toHaveBeenCalledWith('detach_pane', {
    paneId: 'pane-1',
    runId: 'run-1',
  })
})
