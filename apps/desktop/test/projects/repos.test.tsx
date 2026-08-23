import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project repos', () => {
  it('renders repo rows with branch and run state', () => {
    const { getAllByTestId } = render(() => <ProjectsView />)
    const rows = getAllByTestId('project-repo-branch-state')

    expect(rows).toHaveLength(2)
    expect(rows[0].textContent).toContain('agent branches')
  })
})
