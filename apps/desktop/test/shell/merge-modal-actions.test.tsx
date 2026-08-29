import { fireEvent, render, waitFor } from '@solidjs/testing-library'
import { describe, expect, it, vi } from 'vitest'
import { MergeModal } from '../../src/shell/MergeModal'

describe('shell/merge-modal-actions', () => {
  it('runs the merge operation and closes only after it succeeds', async () => {
    let resolveMerge!: () => void
    const onMerge = vi.fn(
      () => new Promise<void>((resolve) => {
        resolveMerge = resolve
      }),
    )
    const onClose = vi.fn()
    const view = render(() => (
      <MergeModal open branch="agent/task-1" onMerge={onMerge} onClose={onClose} />
    ))

    await fireEvent.click(view.getByTestId('merge-confirm'))
    expect(onMerge).toHaveBeenCalledOnce()
    expect(onClose).not.toHaveBeenCalled()

    resolveMerge()
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce())
  })

  it('keeps the modal open and surfaces merge failures', async () => {
    const onMerge = vi.fn(() => Promise.reject(new Error('merge conflict')))
    const onClose = vi.fn()
    const view = render(() => (
      <MergeModal open branch="agent/task-1" onMerge={onMerge} onClose={onClose} />
    ))

    await fireEvent.click(view.getByTestId('merge-confirm'))

    await waitFor(() => expect(view.getByRole('alert').textContent).toContain('merge conflict'))
    expect(onClose).not.toHaveBeenCalled()
    expect(view.queryByTestId('merge-modal')).not.toBeNull()
  })
})
