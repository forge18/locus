import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ViewerStateFamilies } from '../../src/screens/memory/ViewerStateFamilies'

describe('visual: v2 viewers', () => {
  it('declares Light and Dark visual regression coverage', () => {
    const { getByTestId } = render(() => <ViewerStateFamilies />)
    expect(getByTestId('viewer-state-families').getAttribute('data-visual-themes')).toBe('light,dark')
  })
})
