import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop provider selector', () => {
  it('renders preferred aliases and selector preview', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="providers" />)
    expect(getByTestId('provider-model-alias-opus').textContent).toBe('opus')
    expect(getByTestId('provider-selector-preview').textContent).toContain('opus')
  })
})
