import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop desktop route', () => {
  it('identifies the selected-project route', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('develop').getAttribute('data-desktop-route')).toBe('develop')
  })
})
