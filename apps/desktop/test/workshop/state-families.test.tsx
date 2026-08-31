import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('Workshop state families', () => {
  it('keeps fixture controls keyboard-addressable', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="harnesses" />)
    expect(getByTestId('workshop-harnesses').querySelectorAll('button').length).toBeGreaterThan(0)
  })
})
