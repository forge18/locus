import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DevelopView } from '../../src/screens/develop/DevelopView'

describe('Develop review actions', () => {
  it('renders merge, PR, and chunk revert actions', () => {
    const { getByTestId } = render(() => <DevelopView />)
    expect(getByTestId('dev-footer').querySelectorAll('[data-develop-review-action]')).toHaveLength(3)
  })
})
