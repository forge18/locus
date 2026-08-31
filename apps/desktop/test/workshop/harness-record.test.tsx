import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('Workshop harness record', () => {
  it('renders adapter and default configuration', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="harnesses" />)
    expect(getByTestId('harness-adapter-gate').textContent).toContain('built-in')
  })
})
