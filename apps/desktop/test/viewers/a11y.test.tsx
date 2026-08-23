import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ViewerStateFamilies } from '../../src/screens/memory/ViewerStateFamilies'

describe('viewer accessibility', () => {
  it('announces viewer state changes', () => {
    const { getByTestId } = render(() => <ViewerStateFamilies />)
    expect(getByTestId('viewer-state-families').getAttribute('aria-live')).toBe('polite')
  })
})
