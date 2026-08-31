import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('Workshop CLI upload', () => {
  it('renders signature rejection state', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="cli" />)
    expect(getByTestId('cli-upload-verification').textContent).toContain('Minisign')
  })
})
