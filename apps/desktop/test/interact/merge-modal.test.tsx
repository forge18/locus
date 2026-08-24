import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import InteractView from '../../src/screens/interact/InteractView'

describe('interact/merge-modal', () => {
  it('opens the shared merge modal from Commit to branch', () => {
    const { getByTestId, queryByTestId } = render(() => <InteractView />)

    expect(queryByTestId('merge-modal')).toBe(null)
    getByTestId('interact-commit').click()
    expect(getByTestId('merge-modal').textContent).toContain('The work becomes yours.')
    expect(getByTestId('merge-modal').textContent).toContain('interact/r-9f21')
  })

  it('closes without merging when the modal is dismissed', () => {
    const { getByTestId, queryByTestId } = render(() => <InteractView />)

    getByTestId('interact-commit').click()
    getByTestId('merge-modal').querySelector('[aria-label="Close"]')?.dispatchEvent(
      new MouseEvent('click', { bubbles: true }),
    )
    expect(queryByTestId('merge-modal')).toBe(null)
  })
})
