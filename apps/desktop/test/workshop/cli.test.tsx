import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop CLI', () => {
  it('renders categories and image details', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="cli" />)
    expect(getByTestId('cli-group-rust').getAttribute('data-state')).toBe('on')
  })
})
