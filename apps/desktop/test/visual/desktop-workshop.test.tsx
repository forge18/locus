import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('visual Workshop', () => {
  it('renders provider surface in the theme-neutral fixture', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="providers" />)
    expect(getByTestId('workshop-providers')).toBeTruthy()
  })
})
