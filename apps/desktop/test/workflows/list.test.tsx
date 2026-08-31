import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('workflow list', () => {
  it('renders draft and published authoring metadata', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="workflows-list" />)
    expect(getByTestId('workflow-list-published').textContent).toContain('author: Avery')
    expect(getByTestId('workflow-list-draft').textContent).toContain('draft')
  })
})
