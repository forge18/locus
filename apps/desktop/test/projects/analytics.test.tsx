import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project analytics', () => {
  it('renders model, token, cache, and spend totals', () => {
    const { getByTestId } = render(() => <ProjectsView />)
    fireEvent.click(getByTestId('project-tab-analytics'))
    expect(getByTestId('project-analytics-total').textContent).toContain('$1,842')
  })
})
