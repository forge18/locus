import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'
import { configureProjectsStub } from './provider-stub'

describe('project extension groups', () => {
  it('renders group controls', () => {
    const { getByTestId } = render(() => { configureProjectsStub(); return <ProjectsView /> })
    expect(getByTestId('project-extension-groups').textContent).toContain('Extensions')
  })
})
