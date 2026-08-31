import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'
import { configureProjectsStub } from './provider-stub'

describe('project CLI tools', () => {
  it('renders search results and scoped tools', () => {
    const { getByTestId } = render(() => { configureProjectsStub(); return <ProjectsView /> })
    expect(getByTestId('project-cli-tools').textContent).toContain('cargo-nextest')
  })
})
