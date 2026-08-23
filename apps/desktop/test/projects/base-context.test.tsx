import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { ProjectsView } from '../../src/screens/projects/ProjectsView'

describe('project base context', () => {
  it('renders editable context with its token budget meter', () => {
    const { getByTestId } = render(() => <ProjectsView />)
    expect(getByTestId('project-base-context-editor').textContent).toContain('base.md')
    expect(getByTestId('project-base-context-budget').textContent).toContain('1,500 tokens')
  })
})
