import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project extension groups', () => {
  it('renders group controls', () => {
    const { getByTestId } = render(() => <ProjectsView />)
    expect(getByTestId('project-extension-groups').textContent).toContain('Extensions')
  })
})
