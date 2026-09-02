import { invoke } from '@tauri-apps/api/core'

/** Opens another Tauri window; panes never add a second webview to their current window. */
export const detachPane = (paneId: string, runId?: string) =>
  invoke('detach_pane', { paneId, runId })

export const detachedMode = () =>
  new URLSearchParams(window.location.search).get('detached') === 'true'

export const detachedPaneRunId = () =>
  new URLSearchParams(window.location.search).get('run') ?? undefined
