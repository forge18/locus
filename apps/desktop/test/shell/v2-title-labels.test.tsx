import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AppTitleBar } from '../../src/shell/AppTitleBar'

describe('shell/v2-title-labels', () => {
  it('renders the current category and view labels beside the wordmark', () => {
    const { getByTestId } = render(() => <AppTitleBar categoryLabel="Plan" viewLabel="Spec" />)

    expect(getByTestId('title-category').textContent).toBe('Plan')
    expect(getByTestId('title-view').textContent).toBe('Spec')
  })
})
