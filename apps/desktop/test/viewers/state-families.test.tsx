import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ViewerStateFamilies } from '../../src/screens/memory/ViewerStateFamilies'

describe('viewer state families', () => {
  it('renders loading, empty, and error states for all viewers', () => {
    const { getByTestId } = render(() => <ViewerStateFamilies />)
    expect(getByTestId('viewer-state-families').querySelectorAll('[data-viewer-state]')).toHaveLength(24)
  })
})
