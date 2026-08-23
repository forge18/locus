import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop diff modes', () => {
  it('offers split and unified diff controls', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('diff-mode-split').getAttribute('aria-pressed')).toBe('true')
    expect(getByTestId('diff-mode-unified')).toBeTruthy()
  })
})
