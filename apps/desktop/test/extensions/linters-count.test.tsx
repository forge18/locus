import { render, waitFor } from '@solidjs/testing-library'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'

describe('extensions/linters-count', () => {
  beforeEach(() => {
    invoke.mockResolvedValue(3)
  })

  it('loads the linter card count from the core-owned linter directory', async () => {
    const { getByTestId } = render(() => <ExtensionsView onNavigate={() => {}} />)

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('linter_count', { root: '/locus/config/linters' })
      expect(getByTestId('type-count-linters').textContent).toBe('3')
    })
  })
})
