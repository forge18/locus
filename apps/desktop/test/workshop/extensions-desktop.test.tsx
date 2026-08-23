import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ExtensionsView } from '../../src/screens/workshop/ExtensionsView'

describe('Workshop extensions v2', () => {
  it('renders all eight extension types', () => {
    const { getByTestId } = render(() => <ExtensionsView onNavigate={() => {}} />)
    expect(getByTestId('type-grid').querySelectorAll('.type-card')).toHaveLength(8)
  })
})
