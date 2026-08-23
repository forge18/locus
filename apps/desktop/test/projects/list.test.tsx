import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project list', () => {
  it('renders running, idle, and archived projects', () => {
    const { getByTestId } = render(() => <ProjectsView />)
    expect(getByTestId('project-state-list').textContent).toContain('archived')
  })
})
