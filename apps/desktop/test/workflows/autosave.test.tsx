import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('workflow autosave', () => {
  it('exposes authoring save state', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="workflows-visual" />)
    expect(getByTestId('workshop-workflows-visual').textContent).toContain('saved 2s ago')
  })
})
