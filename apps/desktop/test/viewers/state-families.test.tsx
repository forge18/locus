import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ViewerStateFamilies } from '../../src/screens/memory/ViewerStateFamilies'

describe('viewer state families', () => {
  it('renders loading, empty, error, and loaded states for all viewers', () => {
    const { getByTestId } = render(() => <ViewerStateFamilies />)
    const family = getByTestId('viewer-state-families')
    expect(family.querySelectorAll('[data-viewer-state]')).toHaveLength(32)
    expect(family.querySelectorAll('[data-state="loading"]')).toHaveLength(8)
    expect(family.querySelectorAll('[data-state="empty"]')).toHaveLength(8)
    expect(family.querySelectorAll('[data-state="error"]')).toHaveLength(8)
    expect(family.querySelectorAll('[data-state="loaded"]')).toHaveLength(8)
  })
})
