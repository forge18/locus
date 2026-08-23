import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop v2 route', () => {
  it('identifies the selected-project route', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('develop').getAttribute('data-v2-route')).toBe('develop')
  })
})
