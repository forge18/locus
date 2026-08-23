import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project CLI tools', () => {
  it('renders search results and scoped tools', () => {
    const { getByTestId } = render(() => <ProjectsView />)
    expect(getByTestId('project-cli-tools').textContent).toContain('cargo-nextest')
  })
})
